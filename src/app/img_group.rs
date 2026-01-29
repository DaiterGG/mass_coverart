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
    pub fn add_to_group(&mut self, group_id: usize, img_id: usize, imgs: &mut Vec<SongImg>) {
        let new_weight = imgs[img_id].src.get_weight();
        let group = &mut self.groups[group_id];
        group.weight += new_weight;

        let mut new_i = self.groups.len();
        let group = &mut self.groups[group_id].imgs;
        group.push(img_id);

        while new_i > 0 && new_weight > imgs[group[new_i - 1]].src.get_weight() {
            let temp = group[new_i - 1];
            group[new_i - 1] = group[new_i];
            group[new_i] = temp;
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
    fn sort_groups(&mut self, group_id: usize) {
        let mut move_id = group_id;
        while move_id > 0 && self.groups[move_id - 1].weight < self.groups[group_id].weight {
            move_id -= 1;
        }
        if group_id != move_id {
            self.groups.swap(group_id, move_id);
        }
    }
}
