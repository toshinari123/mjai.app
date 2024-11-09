use super::game::{BatchGame, Index};
use super::result::GameResult;
use crate::agent::{new_py_agent, BatchAgent};
use std::fs::{self, File};
use std::io;
use std::iter;
use std::path::PathBuf;

use anyhow::Result;
use flate2::read::GzEncoder;
use flate2::Compression;
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone, Default)]
pub struct MultiMatch {
    pub log_dir: Option<String>,
}

#[pymethods]
impl MultiMatch {
    #[new]
    #[pyo3(signature = (*, log_dir=None))]
    const fn new(log_dir: Option<String>) -> Self {
        Self { log_dir }
    }

    pub fn py_match(
        &self,
        agent1: PyObject,
        agent2: PyObject,
        agent3: PyObject,
        agent4: PyObject,
        seeds: Vec<(u64, u64)>,
        cur_index: u32,
//        rankings: mut Vec<[i32; 4]>,
        py: Python<'_>,
    ) -> Result<[i32; 4]> {
        py.allow_threads(move || {
            let results = self.run_batch(
                |player_ids| new_py_agent(agent1, player_ids),
                |player_ids| new_py_agent(agent2, player_ids),
                |player_ids| new_py_agent(agent3, player_ids),
                |player_ids| new_py_agent(agent4, player_ids),
                seeds, cur_index
            )?;

            let mut rankings = [0; 4];
            for (i, result) in results.iter().enumerate() {
                let rank = result.rankings().rank_by_player[i % 4];
                rankings[rank as usize] += 1;
            }
            Ok(rankings)
        })
    }

    pub fn py_match_continue(
        &self,
        agent1: PyObject,
        agent2: PyObject,
        agent3: PyObject,
        agent4: PyObject,
        scores: [i32; 4],
        kyoku: u8,
        honba: u8,
        kyotaku: u8,
        seeds: Vec<(u64, u64)>,
        cur_index: u32,
//        rankings: mut Vec<[i32; 4]>,
        py: Python<'_>,
    ) -> Result<[i32; 4]> {
        py.allow_threads(move || {
            let results = self.run_batch_continue(
                |player_ids| new_py_agent(agent1, player_ids),
                |player_ids| new_py_agent(agent2, player_ids),
                |player_ids| new_py_agent(agent3, player_ids),
                |player_ids| new_py_agent(agent4, player_ids),
                scores,
                kyoku,
                honba,
                kyotaku,
                seeds,
                cur_index,
            )?;

            let mut rankings = [0; 4];
            for (i, result) in results.iter().enumerate() {
                let rank = result.rankings().rank_by_player[i % 4];
                rankings[rank as usize] += 1;
            }
            Ok(rankings)
        })
    }
}

impl MultiMatch {
    pub fn run_batch<A1, A2, A3, A4>(
        &self,
        new_agent1: A1,
        new_agent2: A2,
        new_agent3: A3,
        new_agent4: A4,
        seeds: Vec<(u64, u64)>,
        cur_index: u32,
//        rankings: mut Vec<[i32; 4]>,
    ) -> Result<Vec<GameResult>>
    where
        A1: FnOnce(&[u8]) -> Result<Box<dyn BatchAgent>>,
        A2: FnOnce(&[u8]) -> Result<Box<dyn BatchAgent>>,
        A3: FnOnce(&[u8]) -> Result<Box<dyn BatchAgent>>,
        A4: FnOnce(&[u8]) -> Result<Box<dyn BatchAgent>>,
    {
        if let Some(dir) = &self.log_dir {
            fs::create_dir_all(dir)?;
        }

        let player_ids: Vec<_> = [0, 1, 2, 3].to_vec();

        let mut agents = [
            new_agent1(&player_ids)?,
            new_agent2(&player_ids)?,
            new_agent3(&player_ids)?,
            new_agent4(&player_ids)?,
        ];

        let indexes = vec![[
            Index {
                agent_idx: 0,
                player_id_idx: 0,
            },
            Index {
                agent_idx: 1,
                player_id_idx: 1,
            },
            Index {
                agent_idx: 2,
                player_id_idx: 2,
            },
            Index {
                agent_idx: 3,
                player_id_idx: 3,
            },
        ]; seeds.len() - cur_index as usize];

        let batch_game = BatchGame::tenhou_hanchan();
        let results = batch_game.run(&mut agents, &indexes[..], &seeds[cur_index as usize..])?;

        if let Some(dir) = &self.log_dir {
            log::info!("dumping game logs");
            for (i, gr) in results.clone().into_iter().enumerate() {
                let (seed, key) = gr.seed;
                let filename: PathBuf = [dir, &format!("{seed}_{key}_{i}_a.json.gz")].iter().collect();
                let log = gr.dump_json_log()?;
                let mut comp = GzEncoder::new(log.as_bytes(), Compression::best());
                let mut f = File::create(filename)?;
                io::copy(&mut comp, &mut f)?;
            }
        }

        Ok(results)
    }

    pub fn run_batch_continue<A1, A2, A3, A4>(
        &self,
        new_agent1: A1,
        new_agent2: A2,
        new_agent3: A3,
        new_agent4: A4,
        scores: [i32; 4],
        kyoku: u8,
        honba: u8,
        kyotaku: u8,
        seeds: Vec<(u64, u64)>,
        cur_index: u32,
//        rankings: mut Vec<[i32; 4]>,
    ) -> Result<Vec<GameResult>>
    where
        A1: FnOnce(&[u8]) -> Result<Box<dyn BatchAgent>>,
        A2: FnOnce(&[u8]) -> Result<Box<dyn BatchAgent>>,
        A3: FnOnce(&[u8]) -> Result<Box<dyn BatchAgent>>,
        A4: FnOnce(&[u8]) -> Result<Box<dyn BatchAgent>>,
    {
        if let Some(dir) = &self.log_dir {
            fs::create_dir_all(dir)?;
        }

        let player_ids: Vec<_> = [0, 1, 2, 3].to_vec();

        let mut agents = [
            new_agent1(&player_ids)?,
            new_agent2(&player_ids)?,
            new_agent3(&player_ids)?,
            new_agent4(&player_ids)?,
        ];

        let indexes = vec![[
            Index {
                agent_idx: 0,
                player_id_idx: 0,
            },
            Index {
                agent_idx: 1,
                player_id_idx: 1,
            },
            Index {
                agent_idx: 2,
                player_id_idx: 2,
            },
            Index {
                agent_idx: 3,
                player_id_idx: 3,
            },
        ]; seeds.len() - cur_index as usize];

        let batch_game = BatchGame::tenhou_hanchan();
        let results = batch_game.run_continue(
            scores,
            kyoku,
            honba,
            kyotaku,
            &mut agents,
            &indexes[..],
            &seeds[cur_index as usize..],
        )?;

        if let Some(dir) = &self.log_dir {
            log::info!("dumping game logs");
            for (i, gr) in results.clone().into_iter().enumerate() {
                let (seed, key) = gr.seed;
                let filename: PathBuf = [dir, &format!("{seed}_{key}_{i}_a.json.gz")].iter().collect();
                let log = gr.dump_json_log()?;
                let mut comp = GzEncoder::new(log.as_bytes(), Compression::best());
                let mut f = File::create(filename)?;
                io::copy(&mut comp, &mut f)?;
            }
        }

        Ok(results)
    }
}

