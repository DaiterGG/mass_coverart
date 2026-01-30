use std::ops::Range;

use log::info;

use crate::app::img::SongImg;

#[derive(Debug, Clone)]
struct ImgGroup {
    weight: i32,
    imgs: Vec<usize>,
}
#[derive(Debug, Clone)]
pub struct ImgGroups {
    groups: Vec<ImgGroup>,
    flat: Vec<usize>,
}
impl ImgGroups {
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            flat: Vec::new(),
        }
    }
    pub fn flat(&self) -> &Vec<usize> {
        &self.flat
    }
    pub fn len(&self) -> usize {
        self.groups.len()
    }
    pub fn first_in_group(&self, group_id: usize) -> usize {
        self.groups[group_id].imgs[0]
    }
    pub fn first_in_first_group(&self) -> usize {
        self.groups[0].imgs[0]
    }
    pub fn clear(&mut self) {
        self.groups.clear();
        self.flat.clear();
    }
    /// * `new_img_id`: id of the img inside imgs
    /// * `imgs`: required to sort inside group
    pub fn add_to_group(
        &mut self,
        group_id: usize,
        new_img: &SongImg,
        new_img_id: usize,
        imgs: &[SongImg],
    ) {
        let new_img_weight = new_img.src.get_weight();
        let group = &mut self.groups[group_id];
        group.weight += new_img_weight;

        let mut new_i = group.imgs.len();
        let group = &mut self.groups[group_id].imgs;
        group.push(new_img_id);

        while new_i > 0 && new_img_weight > imgs[group[new_i - 1]].src.get_weight() {
            group.swap(new_i - 1, new_i);
            new_i -= 1;
        }

        self.sort_groups(group_id);
        self.update_flat();
    }
    pub fn add_new(&mut self, img_id: usize, img_weight: i32) {
        self.groups.push(ImgGroup {
            weight: img_weight,
            imgs: vec![img_id],
        });

        self.sort_groups(self.groups.len() - 1);
        self.update_flat();
    }
    fn sort_groups(&mut self, group_id: usize) {
        let mut move_id = group_id;
        while move_id > 0 && self.groups[move_id - 1].weight < self.groups[group_id].weight {
            move_id -= 1;
        }
        if group_id != move_id {
            let group = self.groups.remove(group_id);
            self.groups.insert(move_id, group);
        }
    }
    /// update flat copy after adding 1 element
    fn update_flat(&mut self) {
        let mut flat_i = 0;
        self.flat.push(0);
        for group in &self.groups {
            for i in &group.imgs {
                self.flat[flat_i] = *i;
                flat_i += 1;
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::{
        api::queue::Source::{BandcampAlbum, LocalFile, YoutubeTitle},
        app::{
            img::{ImageProgress::Preview, ImgFormat::Jpeg, SongImg},
            img_group::{self, ImgGroup, ImgGroups},
        },
    };

    #[test]
    fn sort_g() {
        let mut img_groups = ImgGroups {
            groups: vec![
                ImgGroup {
                    weight: 15,
                    imgs: vec![12],
                },
                ImgGroup {
                    weight: 10,
                    imgs: vec![11],
                },
                ImgGroup {
                    weight: 1000,
                    imgs: vec![13],
                },
            ],
            flat: vec![12, 10],
        };
        img_groups.sort_groups(2);
        assert_eq!(img_groups.groups[0].weight, 1000);
        img_groups.update_flat();
        assert_eq!(img_groups.flat[0], 13);
    }
    #[test]
    fn chain() {
        let mut img_groups = ImgGroups::new();
        let img = SongImg::new(Jpeg, Preview(vec![]), YoutubeTitle, "".to_string());
        let img2 = SongImg::new(Jpeg, Preview(vec![]), BandcampAlbum, "".to_string());
        let img3 = SongImg::new(Jpeg, Preview(vec![]), LocalFile, "".to_string());
        let img4 = SongImg::new(Jpeg, Preview(vec![]), YoutubeTitle, "".to_string());
        let mut imgs = vec![];
        img_groups.add_new(0, 10);
        imgs.push(img);

        img_groups.add_new(1, 15);
        imgs.push(img2);

        let flat = img_groups.flat();
        assert_eq!((flat[0], flat[1]), (1, 0));

        img_groups.add_new(2, 99999);
        imgs.push(img3);

        let flat = img_groups.flat();
        assert_eq!((flat[0], flat[1], flat[2]), (2, 1, 0));
        img_groups.add_to_group(1, &img4, imgs.len(), &imgs);
        imgs.push(img4);
    }
}
