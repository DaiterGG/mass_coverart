use std::{
    io::{Cursor, Read},
    sync::Arc,
    thread::{self, yield_now},
    time::{Duration, Instant},
};

use anyhow::bail;
use bytes::Bytes;
use iced::widget::image::Handle;
use image::{GenericImageView, ImageReader, Rgb};
use image_compare::{
    Algorithm::MSSIMSimple, Metric, gray_similarity_histogram, gray_similarity_structure,
};
use tokio::{sync::Semaphore, time::interval};

use crate::{
    api::queue::ReturnSongImg,
    app::{song::GroupSize, song_img::SongImg},
};

const THRESHOLD: f64 = 0.3;
const PREVIEW_DIM: u32 = 100;
pub async fn decode(
    mut new: ReturnSongImg,
    sem: Arc<Semaphore>,
) -> Result<ReturnSongImg, anyhow::Error> {
    let permit = sem.acquire().await.unwrap();
    dbg!(sem.available_permits());
    let b = ImageReader::with_format(Cursor::new(new.img.bytes()), new.img.format.convert())
        .decode()?;

    yield_now();

    new.img.set_resolution(b.dimensions());
    let dyn_img = b.resize_to_fill(
        PREVIEW_DIM,
        PREVIEW_DIM,
        image::imageops::FilterType::Nearest,
    );
    let dyn_clone = dyn_img.clone();
    yield_now();

    let prev = dyn_clone.into_rgba8().into_vec();
    let prev = Bytes::from_owner(prev);
    new.img.preview = Some(Handle::from_rgba(PREVIEW_DIM, PREVIEW_DIM, prev));

    let samp = dyn_img.clone().into_luma8();
    new.img.sort_sample = Some(samp);

    drop(permit);
    Ok(new)
}
pub fn push_and_group(
    groups: &mut Vec<GroupSize>,
    all: &mut Vec<SongImg>,
    mut new: SongImg,
) -> Result<(), anyhow::Error> {
    let b = new.sort_sample.unwrap();

    // let (b_w, b_h) = b.dimensions();
    // iterate over first element of each group
    let mut end_of_groups = 0;
    let mut group = 0;
    let mut i = 0;
    while i < all.len() {
        let a = all[i].sort_sample.as_ref().unwrap();

        let score = gray_similarity_structure(&MSSIMSimple, a, &b)?.score;

        if score > THRESHOLD {
            break;
        }

        if group < groups.len() {
            i += groups[group] as usize;
            end_of_groups += groups[group] as usize;
            group += 1;
        } else {
            i += 1;
        }
    }
    new.sort_sample = Some(b);

    // not found / empty
    if i == all.len() {
        all.push(new);
        return Ok(());
    }

    let mut group_first_element = i;
    // within existing group
    if group < groups.len() {
        // group is index groug to insert into

        // rearrange groups until sorted by group size
        while group > 0 && groups[group] + 1 > groups[group - 1] {
            // swap each element
            for group_i in 0..groups[group] as usize {
                all.swap(
                    group_first_element + group_i,
                    group_first_element + group_i - groups[group] as usize,
                );
            }
            // update to point to swapped group
            group_first_element -= groups[group] as usize;
            group -= 1;
        }
        let mut group_i = 0;
        while group_i < groups[group] as usize
            && all[group_first_element + group_i].resolution().1 > new.resolution().1
        {
            group_i += 1;
        }
        groups[group] += 1;
        all.insert(group_first_element + group_i, new);
        return Ok(());
    }
    // form new group
    all.swap(end_of_groups, group_first_element);
    if all[end_of_groups].resolution().1 > new.resolution().1 {
        all.insert(end_of_groups + 1, new);
    } else {
        all.insert(end_of_groups, new);
    }
    groups.push(2);

    Ok(())
}
