// pub mod single_hero_skill_study;
pub mod static_duo_skill_study;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    equipment::Blueprint,
    hero_builder::HeroClass,
    skills::{HeroSkill, InnateSkill},
};

extern crate csv;

/// Defines a plan for generating and ranking Trials
/// A trial is run for each permutation of team/dungeon variation
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Study {
    identifier: String,
    description: String,
    simulation_qty: i32,
    runoff_scoring_threshold: f64, // The top X% of the results will be re-tested on the n+1 dungeon in the dungeons vec until either there are no successes or the vec is exhausted. Pass 100.0 to disable runoff scoring
    status: StudyStatus,
    hero_builder_information: HeroBuilderInformation,
}

pub fn create_study(
    identifier: String,
    description: String,
    simulation_qty: i32,
    runoff_scoring_threshold: f64,
    hero_builder_information: HeroBuilderInformation,
) -> Study {
    return Study {
        identifier,
        description,
        simulation_qty,
        runoff_scoring_threshold,
        status: StudyStatus::Created,
        hero_builder_information,
    };
}

/// Runnable studies must have a run function
pub trait Runnable {
    fn run(&mut self);
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum StudyStatus {
    Created,
    Running,
    Finished,
}

/// Defines a holder for hero builder information necessary to create each variation of the subject hero(es)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HeroBuilderInformation {
    pub bp_map: HashMap<String, Blueprint>,
    pub hero_classes: HashMap<String, HeroClass>,
    pub hero_skill_tier_1_name_map: HashMap<String, String>,
    pub hero_skill_map: HashMap<String, HeroSkill>,
    pub class_innate_skill_names_map: HashMap<String, String>,
    pub innate_skill_map: HashMap<String, InnateSkill>,
}
