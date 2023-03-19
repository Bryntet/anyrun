use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::{anyrun_interface::HandleResult, *};
use fuzzy_matcher::FuzzyMatcher;
use kidex_common::IndexEntry;
use std::process::Command;

pub fn handler(selection: Match, index: &mut Vec<(usize, IndexEntry)>) -> HandleResult {
    let (_, index_entry) = index
        .iter()
        .find(|(id, _)| selection.id == ROption::RSome(*id as u64))
        .unwrap();

    if let Err(why) = Command::new("xdg-open").arg(&index_entry.path).spawn() {
        println!("Error running xdg-open: {}", why);
    }

    HandleResult::Close
}

pub fn init(_config_dir: RString) -> Vec<(usize, IndexEntry)> {
    match kidex_common::util::get_index(None) {
        Ok(index) => index.into_iter().enumerate().collect(),
        Err(why) => {
            println!("Failed to get kidex index: {}", why);
            Vec::new()
        }
    }
}

pub fn get_matches(input: RString, index: &mut Vec<(usize, IndexEntry)>) -> RVec<Match> {
    let matcher = fuzzy_matcher::skim::SkimMatcherV2::default().smart_case();
    let mut index = index
        .clone()
        .into_iter()
        .filter_map(|(id, index_entry)| {
            matcher
                .fuzzy_match(&index_entry.path.as_os_str().to_string_lossy(), &input)
                .map(|val| (index_entry, id, val))
        })
        .collect::<Vec<_>>();

    index.sort_by(|a, b| b.2.cmp(&a.2));

    index.truncate(3);
    index
        .into_iter()
        .map(|(entry_index, id, _)| Match {
            title: entry_index
                .path
                .file_name()
                .map(|name| name.to_string_lossy().into())
                .unwrap_or("N/A".into()),
            icon: ROption::RSome(if entry_index.directory {
                "folder".into()
            } else {
                "text-x-generic".into()
            }),
            description: entry_index
                .path
                .parent()
                .map(|path| path.display().to_string().into())
                .into(),
            id: ROption::RSome(id as u64),
        })
        .collect()
}

pub fn info() -> PluginInfo {
    PluginInfo {
        name: "Kidex".into(),
        icon: "folder".into(),
    }
}

plugin!(init, info, get_matches, handler, Vec<(usize, IndexEntry)>);
