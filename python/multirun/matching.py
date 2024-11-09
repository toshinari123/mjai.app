import json
import random
import uuid
import copy
from collections import Counter
from pathlib import Path
from typing import Any, TypeAlias

from loguru import logger
from mjai.elo import update_multi_players_elo
from multirun.game import MultiSimulator

UserId: TypeAlias = int
LogId: TypeAlias = str
DuplicateGame: TypeAlias = tuple[int, str, list[UserId]]

class MultiMatchingWithoutElo:
    def __init__(
        self,
        path_map: dict[UserId, Path],
        nummatches = 100,
    ):
        self.path_map = path_map
        self.name_dict = {
            user_id: path.stem for user_id, path in path_map.items()
        }
        self.ids = [user_id for user_id, _ in self.path_map.items()]
        self.match_count: Counter = Counter(
            {user_id: 0 for user_id in self.ids}
        )
        self.nummatches = nummatches

    def save_match_json(
        self, batch_date: str, matching_json: list[dict[str, Any]]
    ):
        matching_json_path = Path(f"./matching/{batch_date}.json")
        matching_json_path.parent.mkdir(parents=True, exist_ok=True)
        json.dump(matching_json, matching_json_path.open("w"))

    def match(
        self, batch_date: str
    ) -> dict[str, Any]:
        target_player_id = self.get_target_player()
        user_id_list = self.get_new_match_tuple(target_player_id)
        duplicate_game = self.get_duplicate_game(user_id_list)
        seeds = [];
        for i in range(self.nummatches):
            seeds.append(self.get_random_seed())

        matching_detail = {}

        try:
            for dup_idx, log_id, user_id_list_ in duplicate_game:
                filepath_list = [
                    self.path_map[user_id] for user_id in user_id_list_
                ]
                logger.info(f"Start games {log_id} (dup_idx={dup_idx})")
                MultiSimulator(
                    filepath_list,
                    logs_dir=f"./logs/{batch_date}/{log_id}",
                    seeds=copy.deepcopy(seeds),
                ).run()

            matching_detail = self.collect_duplicate_game_result(
                duplicate_game, batch_date, copy.deepcopy(seeds)
            )
            for match_user_id in user_id_list:
                self.match_count[match_user_id] += 1

        except Exception as e:
            logger.error(f"Unexpected error. {str(e)}")

        return matching_detail

    def get_random_seed(self) -> tuple[int, int]:
        seed_nonce = random.randint(1, 100000)
        seed_key = random.randint(1, 100000)
        return (seed_nonce, seed_key)

    def collect_duplicate_game_result(
        self,
        duplicate_game: list[DuplicateGame],
        batch_date: str,
        seeds: list[tuple[int, int]],
    ) -> dict[str, Any]:
        user_id_list = duplicate_game[0][2]

        summary_errors = []
        matches = []
        for dup_idx, log_id, dup_user_id_list in duplicate_game:
            summary_data = json.load(
                Path(f"./logs/{batch_date}/{log_id}/summary.json").open("r")
            )
            match_info = {
                "log_id": log_id,
                "user_ids": dup_user_id_list,
                "ranks": summary_data["rank"],
            }
            error_data = json.load(
                Path(f"./logs/{batch_date}/{log_id}/errors.json").open("r")
            )
            for error_info in error_data:
                if "player_id" in error_info:
                    summary_errors.append(
                        {
                            "duplication_index": dup_idx,
                            "player_id": error_info["player_id"],
                            "user_id": dup_user_id_list[
                                error_info["player_id"]
                            ],
                        }
                    )
            matches.append(match_info)

        return {
            "seed_values": seeds,
            "users": user_id_list,
            "matches": matches,
            "errors": summary_errors,
        }

    def get_duplicate_game(
        self, user_ids: list[UserId]
    ) -> list[DuplicateGame]:
        return [
            (0, str(uuid.uuid4()), user_ids),
            (1, str(uuid.uuid4()), user_ids[1:] + user_ids[:1]),
            (2, str(uuid.uuid4()), user_ids[2:] + user_ids[:2]),
            (3, str(uuid.uuid4()), user_ids[3:] + user_ids[:3]),
        ]

    def get_target_player(self) -> UserId:
        # Get lowest frequency player
        return self.match_count.most_common()[-1][0]

    def get_new_match_tuple(self, target_user_id: UserId) -> list[UserId]:
        candidate_ids = self.ids;
        candidate_ids.remove(target_user_id)
        matched_user_ids = [target_user_id]

        for _ in range(3):
            choice = random.choices(candidate_ids, k=1)
            chosen_user_id = choice[0]
            candidate_ids.remove(chosen_user_id)
            matched_user_ids.append(chosen_user_id)
        assert len(matched_user_ids) == 4
        return matched_user_ids
