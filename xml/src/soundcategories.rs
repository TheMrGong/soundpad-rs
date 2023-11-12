use crate::{soundlist::empty_string_as_none};
use serde::Deserialize;
use std::str::FromStr;

#[derive(Deserialize, Debug, Clone)]
pub struct SoundListAndCategories {
    #[serde(rename = "$value")]
    pub sounds_or_categories: Vec<SoundOrCategories>,
}

impl FromStr for SoundListAndCategories {
    type Err = serde_xml_rs::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_xml_rs::from_str(s)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub enum SoundOrCategories {
    Sound(Sound),
    Categories(Categories),
    Hotbar(Hotbar)
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Categories {
    #[serde(rename = "$value")]
    pub categories: Vec<Category>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    #[serde(rename = "$value")]
    pub sounds_and_categories: Option<Vec<SoundOrCategory>>,

    #[serde(rename = "type")]
    pub r#type: Option<String>,
    pub hidden: Option<bool>,
    pub name: Option<String>,
    #[serde(with = "empty_string_as_none")]
    pub icon: Option<String>,
    pub index: Option<u64>,
    pub key_modifiers: Option<u64>,
    pub key: Option<u64>,
}

#[derive(Deserialize, Debug, Clone)]
pub enum SoundOrCategory {
    Category(Category),
    Sound(CategorySound)
}

#[derive(Deserialize, Debug, Clone)]
pub struct Sound {
    pub url: String
}
#[derive(Deserialize, Debug, Clone)]
pub struct Hotbar {

}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CategorySound {
    pub id: u64,
}

#[cfg(test)]
mod tests {

    use super::*;

    const MYLIST: &str = include_str!("SoundListWithCategories.xml");

    fn get_categories() -> Option<Vec<Category>> {
        let deserialized: SoundListAndCategories = serde_xml_rs::from_str(MYLIST).unwrap();

        deserialized
            .sounds_or_categories
            .into_iter()
            .find_map(|it| {
                if let SoundOrCategories::Categories(categories) = it {
                    Some(categories.categories)
                } else {
                    None
                }
            })
    }

    #[test]
    fn categories() {
        let categories = get_categories().unwrap();
        let category = categories
            .iter()
            .find(|category| category.name.as_ref().is_some_and(|name| name.eq("Nom")))
            .expect("Nom category");

        assert_eq!(4, category.sounds_and_categories.as_ref().unwrap().into_iter().filter(|it| matches!(it, SoundOrCategory::Sound(_))).count());
    }

    #[test]
    fn sub_categories() {
        let categories = get_categories().unwrap();
        let category = categories
            .iter()
            .find(|category| category.name.as_ref().is_some_and(|name| name.eq("Nom")))
            .expect("Nom category");
        let sub_nom = category.sounds_and_categories.as_ref().unwrap().into_iter().find_map(|sound_or_cat| {
            if let SoundOrCategory::Category(category) = sound_or_cat {
                if category.name.as_ref().is_some_and(|name| name.eq("Sub-Nom")) {
                    return Some(category);
                }
            }
            return None;
        }).unwrap();

        assert_eq!(5, sub_nom.sounds_and_categories.as_ref().unwrap().into_iter().filter(|it| matches!(it, SoundOrCategory::Sound(_))).count());
    }
}
