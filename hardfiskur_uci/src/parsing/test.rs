use std::time::Duration;

use hardfiskur_core::board::{PieceType, Square, UCIMove};

use crate::{
    uci_message::ProtectionState, UCIInfo, UCIInfoCurrLine, UCIInfoScore, UCIMessage,
    UCIOptionConfig, UCIPosition, UCIPositionBase, UCISearchControl, UCITimeControl,
};

use pretty_assertions::assert_eq;

#[test]
fn parse_empty_should_fail() {
    "".parse::<UCIMessage>()
        .expect_err("should not parse empty string");
}

#[test]
fn parse_nl() {
    "\n".parse::<UCIMessage>()
        .expect_err("should not parse new line");
}

#[test]
fn parse_uci() {
    let msg: UCIMessage = "uci".parse().unwrap();
    assert_eq!(msg, UCIMessage::UCI);
}

#[test]
fn parse_uci_accepts_whitespace_and_newlines() {
    let msg: UCIMessage = "\tuci\n".parse().unwrap();
    assert_eq!(msg, UCIMessage::UCI);

    let msg: UCIMessage = "    uci\r\n".parse().unwrap();
    assert_eq!(msg, UCIMessage::UCI);
}

#[test]
fn parse_debug() {
    let msg: UCIMessage = "debug".parse().unwrap();
    assert_eq!(msg, UCIMessage::Debug(true));
}

#[test]
fn parse_debug_on() {
    let msg: UCIMessage = "debug on".parse().unwrap();
    assert_eq!(msg, UCIMessage::Debug(true));
}

#[test]
fn parse_debug_off() {
    let msg: UCIMessage = "debug off".parse().unwrap();
    assert_eq!(msg, UCIMessage::Debug(false));
}

#[test]
fn parse_ignores_unknown_preceding_tokens() {
    let msg: UCIMessage = "joho debug on".parse().unwrap();
    assert_eq!(msg, UCIMessage::Debug(true));

    let msg: UCIMessage = "asdf jkl; - - - ?? debug on".parse().unwrap();
    assert_eq!(msg, UCIMessage::Debug(true));
}

#[test]
fn parse_ignores_unknown_following_tokens() {
    let msg: UCIMessage = "debug on joho".parse().unwrap();
    assert_eq!(msg, UCIMessage::Debug(true));

    let msg: UCIMessage = "debug on -12 3 9014 2831@~!".parse().unwrap();
    assert_eq!(msg, UCIMessage::Debug(true));
}

#[test]
fn parse_isready() {
    let msg: UCIMessage = "isready".parse().unwrap();
    assert_eq!(msg, UCIMessage::IsReady);
}

#[test]
fn parse_setoption_name_and_value() {
    let msg: UCIMessage = "setoption name Nullmove value true".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::SetOption {
            name: "Nullmove".to_string(),
            value: Some("true".to_string())
        }
    );
}

#[test]
fn parse_setoption_name_and_value_with_whitespace() {
    let msg: UCIMessage = "setoption name Hash  Size value\t1000\tMiB"
        .parse()
        .unwrap();
    assert_eq!(
        msg,
        UCIMessage::SetOption {
            name: "Hash  Size".to_string(),
            value: Some("1000\tMiB".to_string())
        }
    );
}

#[test]
fn parse_setoption_name_only() {
    let msg: UCIMessage = "setoption name Clear Hash".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::SetOption {
            name: "Clear Hash".to_string(),
            value: None
        }
    );
}

#[test]
fn parse_register_later() {
    let msg: UCIMessage = "register later".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Register {
            later: true,
            name: None,
            code: None
        }
    );
}

#[test]
fn parse_register_name_and_code() {
    let msg: UCIMessage = "register name This is my name  -  Stefan MK code my name is the same as my code\tStefan MK".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Register {
            later: false,
            name: Some("This is my name  -  Stefan MK".to_string()),
            code: Some("my name is the same as my code\tStefan MK".to_string())
        }
    );
}

#[test]
fn parse_register_name_only() {
    let msg: UCIMessage = "register name This is my name  -  Stefan MK"
        .parse()
        .unwrap();
    assert_eq!(
        msg,
        UCIMessage::Register {
            later: false,
            name: Some("This is my name  -  Stefan MK".to_string()),
            code: None,
        }
    );
}

#[test]
fn parse_register_code_only() {
    let msg: UCIMessage = "register code my name is the same as my code\tStefan MK"
        .parse()
        .unwrap();
    assert_eq!(
        msg,
        UCIMessage::Register {
            later: false,
            name: None,
            code: Some("my name is the same as my code\tStefan MK".to_string())
        }
    );
}

#[test]
fn parse_uci_new_game() {
    let msg: UCIMessage = "ucinewgame".parse().unwrap();
    assert_eq!(msg, UCIMessage::UCINewGame);
}

#[test]
fn parse_position_startpos() {
    let msg: UCIMessage = "position startpos".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Position(UCIPosition {
            base: UCIPositionBase::StartPos,
            moves: vec![]
        })
    );
}

#[test]
fn parse_position_fen() {
    let msg: UCIMessage = "position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        .parse()
        .unwrap();
    assert_eq!(
        msg,
        UCIMessage::Position(UCIPosition {
            base: UCIPositionBase::Fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()
            ),
            moves: vec![]
        })
    );
}

#[test]
fn parse_position_startpos_with_moves() {
    let msg: UCIMessage = "position startpos moves e2e4 e1g1 b7c8q".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Position(UCIPosition {
            base: UCIPositionBase::StartPos,
            moves: vec![
                UCIMove {
                    from: Square::E2,
                    to: Square::E4,
                    promotion: None
                },
                UCIMove {
                    from: Square::E1,
                    to: Square::G1,
                    promotion: None
                },
                UCIMove {
                    from: Square::B7,
                    to: Square::C8,
                    promotion: Some(PieceType::Queen)
                },
            ]
        })
    );
}

#[test]
fn parse_position_fen_with_moves() {
    let msg: UCIMessage = "position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 moves e2e4 e1g1 b7c8q".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Position(UCIPosition {
            base: UCIPositionBase::Fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()
            ),
            moves: vec![
                UCIMove {
                    from: Square::E2,
                    to: Square::E4,
                    promotion: None
                },
                UCIMove {
                    from: Square::E1,
                    to: Square::G1,
                    promotion: None
                },
                UCIMove {
                    from: Square::B7,
                    to: Square::C8,
                    promotion: Some(PieceType::Queen)
                },
            ]
        })
    );
}

#[test]
fn parse_go() {
    let msg: UCIMessage = "go".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Go {
            time_control: None,
            search_control: None
        }
    );
}

#[test]
fn parse_go_ponder() {
    let msg: UCIMessage = "go ponder".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Go {
            time_control: Some(UCITimeControl::Ponder),
            search_control: None
        }
    );
}

#[test]
fn parse_go_infinite() {
    let msg: UCIMessage = "go infinite".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Go {
            time_control: Some(UCITimeControl::Infinite),
            search_control: None
        }
    );
}

#[test]
fn parse_go_movetime() {
    let msg: UCIMessage = "go movetime 1234".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Go {
            time_control: Some(UCITimeControl::MoveTime(Duration::from_millis(1234))),
            search_control: None
        }
    );
}

#[test]
fn parse_go_timeleft() {
    let msg: UCIMessage = "go wtime 59000 btime 58000 winc 1000 binc 2000 movestogo 21"
        .parse()
        .unwrap();
    assert_eq!(
        msg,
        UCIMessage::Go {
            time_control: Some(UCITimeControl::TimeLeft {
                white_time: Some(Duration::from_secs(59)),
                black_time: Some(Duration::from_secs(58)),
                white_increment: Some(Duration::from_secs(1)),
                black_increment: Some(Duration::from_secs(2)),
                moves_to_go: Some(21),
            }),
            search_control: None
        }
    );
}

#[test]
fn parse_go_search_control() {
    let msg: UCIMessage = "go searchmoves e2e4 e7e6 depth 6 nodes 98765 mate 3"
        .parse()
        .unwrap();
    assert_eq!(
        msg,
        UCIMessage::Go {
            time_control: None,
            search_control: Some(UCISearchControl {
                search_moves: vec!["e2e4".parse().unwrap(), "e7e6".parse().unwrap()],
                depth: Some(6),
                nodes: Some(98765),
                mate: Some(3),
            })
        }
    );
}

#[test]
fn parse_go_arbitrary_option_order() {
    let msg: UCIMessage = "go mate 3 winc 1000 depth 6 nodes 98765 btime 58000 searchmoves e2e4 e7e6 binc 2000 wtime 59000 nodes 98765"
        .parse()
        .unwrap();
    assert_eq!(
        msg,
        UCIMessage::Go {
            time_control: Some(UCITimeControl::TimeLeft {
                white_time: Some(Duration::from_secs(59)),
                black_time: Some(Duration::from_secs(58)),
                white_increment: Some(Duration::from_secs(1)),
                black_increment: Some(Duration::from_secs(2)),
                moves_to_go: None,
            }),
            search_control: Some(UCISearchControl {
                search_moves: vec!["e2e4".parse().unwrap(), "e7e6".parse().unwrap()],
                depth: Some(6),
                nodes: Some(98765),
                mate: Some(3),
            })
        }
    );
}

#[test]
fn parse_go_negative_times_ignored() {
    // cutechess allows players to go over time and then returns the times as
    // negative numbers.
    let msg: UCIMessage = "go mate 3 winc 1000 depth 6 nodes 98765 btime -58000 searchmoves e2e4 e7e6 binc 2000 wtime 59000 nodes 98765"
        .parse()
        .unwrap();
    assert_eq!(
        msg,
        UCIMessage::Go {
            time_control: Some(UCITimeControl::TimeLeft {
                white_time: Some(Duration::from_secs(59)),
                black_time: None,
                white_increment: Some(Duration::from_secs(1)),
                black_increment: Some(Duration::from_secs(2)),
                moves_to_go: None,
            }),
            search_control: Some(UCISearchControl {
                search_moves: vec!["e2e4".parse().unwrap(), "e7e6".parse().unwrap()],
                depth: Some(6),
                nodes: Some(98765),
                mate: Some(3),
            })
        }
    );
}

#[test]
fn parse_stop() {
    let msg: UCIMessage = "stop".parse().unwrap();
    assert_eq!(msg, UCIMessage::Stop);
}

#[test]
fn parse_ponderhit() {
    let msg: UCIMessage = "ponderhit".parse().unwrap();
    assert_eq!(msg, UCIMessage::PonderHit);
}

#[test]
fn parse_quit() {
    let msg: UCIMessage = "quit".parse().unwrap();
    assert_eq!(msg, UCIMessage::Quit);
}

#[test]
fn parse_id_name() {
    let msg: UCIMessage = "id name Shredder X.Y".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Id {
            name: Some("Shredder X.Y".to_string()),
            author: None
        }
    );
}

#[test]
fn parse_id_author() {
    let msg: UCIMessage = "    id   author\tStefan MK\r\n".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Id {
            name: None,
            author: Some("Stefan MK".to_string()),
        }
    );
}

#[test]
fn parse_uciok() {
    let msg: UCIMessage = "uciok".parse().unwrap();
    assert_eq!(msg, UCIMessage::UCIOk);
}

#[test]
fn parse_readyok() {
    let msg: UCIMessage = "readyok".parse().unwrap();
    assert_eq!(msg, UCIMessage::ReadyOk);
}

#[test]
fn parse_bestmove() {
    let msg: UCIMessage = "bestmove a1d4".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::BestMove {
            best_move: UCIMove {
                from: Square::A1,
                to: Square::D4,
                promotion: None
            },
            ponder: None
        }
    );
}

#[test]
fn parse_bestmove_ponder() {
    let msg: UCIMessage = "bestmove a1d4   ponder    e6e5".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::BestMove {
            best_move: UCIMove {
                from: Square::A1,
                to: Square::D4,
                promotion: None
            },
            ponder: Some(UCIMove {
                from: Square::E6,
                to: Square::E5,
                promotion: None
            })
        }
    );
}

#[test]
fn parse_copyprotection() {
    let msg: UCIMessage = "copyprotection checking".parse().unwrap();
    assert_eq!(msg, UCIMessage::CopyProtection(ProtectionState::Checking));

    let msg: UCIMessage = "copyprotection   ok".parse().unwrap();
    assert_eq!(msg, UCIMessage::CopyProtection(ProtectionState::Ok));

    let msg: UCIMessage = "copyprotection     error\r".parse().unwrap();
    assert_eq!(msg, UCIMessage::CopyProtection(ProtectionState::Error));
}

#[test]
fn parse_registration() {
    let msg: UCIMessage = "registration checking".parse().unwrap();
    assert_eq!(msg, UCIMessage::Registration(ProtectionState::Checking));

    let msg: UCIMessage = "registration   ok".parse().unwrap();
    assert_eq!(msg, UCIMessage::Registration(ProtectionState::Ok));

    let msg: UCIMessage = "registration     error\r".parse().unwrap();
    assert_eq!(msg, UCIMessage::Registration(ProtectionState::Error));
}

#[test]
fn parse_info_depth() {
    let msg: UCIMessage = "info depth 2".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            depth: Some(2),
            ..Default::default()
        })
    );
}

#[test]
fn parse_info_seldepth() {
    let msg: UCIMessage = "info seldepth 3".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            sel_depth: Some(3),
            ..Default::default()
        })
    );
}

#[test]
fn parse_info_time() {
    let msg: UCIMessage = "info time 12005".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            time: Some(Duration::from_millis(12005)),
            ..Default::default()
        })
    );
}

#[test]
fn parse_info_nodes() {
    let msg: UCIMessage = "info nodes 48929".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            nodes: Some(48929),
            ..Default::default()
        })
    );
}

#[test]
fn parse_info_pv() {
    let msg: UCIMessage = "info pv e2e4 e7e5 g1f3".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            pv: vec![
                "e2e4".parse().unwrap(),
                "e7e5".parse().unwrap(),
                "g1f3".parse().unwrap()
            ],
            ..Default::default()
        })
    );
}

#[test]
fn parse_info_multipv() {
    let msg: UCIMessage = "info multipv 3".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            multi_pv: Some(3),
            ..Default::default()
        })
    );
}

#[test]
fn parse_info_score_cp() {
    let msg: UCIMessage = "info score cp +123".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            score: Some(UCIInfoScore {
                cp: Some(123),
                ..Default::default()
            }),
            ..Default::default()
        })
    );
}

#[test]
fn parse_info_score_mate() {
    let msg: UCIMessage = "info score mate -7".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            score: Some(UCIInfoScore {
                mate: Some(-7),
                ..Default::default()
            }),
            ..Default::default()
        })
    );
}

#[test]
fn parse_info_score_lowerbound() {
    let msg: UCIMessage = "info score cp -561 lowerbound".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            score: Some(UCIInfoScore {
                cp: Some(-561),
                lower_bound: true,
                ..Default::default()
            }),
            ..Default::default()
        })
    );
}

#[test]
fn parse_info_score_upperbound() {
    let msg: UCIMessage = "info score cp 12 upperbound".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            score: Some(UCIInfoScore {
                cp: Some(12),
                upper_bound: true,
                ..Default::default()
            }),
            ..Default::default()
        })
    );
}

#[test]
fn parse_info_currmove() {
    let msg: UCIMessage = "info currmove e2e4".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            curr_move: Some("e2e4".parse().unwrap()),
            ..Default::default()
        })
    );
}

#[test]
fn parse_info_currmovenumber() {
    let msg: UCIMessage = "info currmovenumber 4".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            curr_move_number: Some(4),
            ..Default::default()
        })
    );
}

#[test]
fn parse_info_hashfull() {
    let msg: UCIMessage = "info hashfull 121".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            hash_full: Some(121),
            ..Default::default()
        })
    );
}

#[test]
fn parse_info_nps() {
    let msg: UCIMessage = "info nps 576000".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            nps: Some(576000),
            ..Default::default()
        })
    );
}

#[test]
fn parse_info_tbhits() {
    let msg: UCIMessage = "info tbhits 83829".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            tb_hits: Some(83829),
            ..Default::default()
        })
    );
}

#[test]
fn parse_info_sbhits() {
    let msg: UCIMessage = "info sbhits 1003".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            sb_hits: Some(1003),
            ..Default::default()
        })
    );
}

#[test]
fn parse_info_cpuload() {
    let msg: UCIMessage = "info cpuload 488".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            cpu_load: Some(488),
            ..Default::default()
        })
    );
}

#[test]
fn parse_info_string() {
    let msg: UCIMessage = "info string This is my helpful info string"
        .parse()
        .unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            string: Some("This is my helpful info string".to_string()),
            ..Default::default()
        })
    );
}

#[test]
fn parse_info_string_goes_until_eof() {
    let msg: UCIMessage = "info nodes 1 string looks like another option: depth 2 currmove e2e4"
        .parse()
        .unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            nodes: Some(1),
            string: Some("looks like another option: depth 2 currmove e2e4".to_string()),
            ..Default::default()
        })
    );
}

#[test]
fn parse_info_refutation() {
    let msg: UCIMessage = "info refutation d1h5 g8h5".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            refutation: vec!["d1h5".parse().unwrap(), "g8h5".parse().unwrap()],
            ..Default::default()
        })
    );
}

#[test]
fn parse_info_currline() {
    let msg: UCIMessage = "info currline d1h5 g8h5".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            curr_line: Some(UCIInfoCurrLine {
                cpu_nr: None,
                moves: vec!["d1h5".parse().unwrap(), "g8h5".parse().unwrap()]
            }),
            ..Default::default()
        })
    );
}
#[test]
fn parse_info_misc() {
    let msg: UCIMessage = "info score cp 20  depth 3 nodes 423 time 15 pv f1c4 g8f6 b1c3 "
        .parse()
        .unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            score: Some(UCIInfoScore {
                cp: Some(20),
                ..Default::default()
            }),
            depth: Some(3),
            nodes: Some(423),
            time: Some(Duration::from_millis(15)),
            pv: vec![
                "f1c4".parse().unwrap(),
                "g8f6".parse().unwrap(),
                "b1c3".parse().unwrap()
            ],
            ..Default::default()
        })
    );
}

#[test]
fn parse_info_currline_with_cpu_nr() {
    let msg: UCIMessage = "info currline  6 d1h5 g8h5".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Info(UCIInfo {
            curr_line: Some(UCIInfoCurrLine {
                cpu_nr: Some(6),
                moves: vec!["d1h5".parse().unwrap(), "g8h5".parse().unwrap()]
            }),
            ..Default::default()
        })
    );
}

#[test]
fn parse_option_check() {
    let msg: UCIMessage = "option name All the keywords check default min max var type check"
        .parse()
        .unwrap();
    assert_eq!(
        msg,
        UCIMessage::Option(UCIOptionConfig::Check {
            name: "All the keywords check default min max var".to_string(),
            default: None
        })
    );
}

#[test]
fn parse_option_check_default() {
    let msg: UCIMessage = "option name My Checkbox type check default true"
        .parse()
        .unwrap();
    assert_eq!(
        msg,
        UCIMessage::Option(UCIOptionConfig::Check {
            name: "My Checkbox".to_string(),
            default: Some(true)
        })
    );

    let msg: UCIMessage = "option name My Checkbox type check default false"
        .parse()
        .unwrap();
    assert_eq!(
        msg,
        UCIMessage::Option(UCIOptionConfig::Check {
            name: "My Checkbox".to_string(),
            default: Some(false)
        })
    );
}

#[test]
fn parse_option_spin() {
    let msg: UCIMessage = "option name Spinner type spin".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Option(UCIOptionConfig::Spin {
            name: "Spinner".to_string(),
            default: None,
            min: None,
            max: None,
        })
    );
}

#[test]
fn parse_option_spin_min_only() {
    let msg: UCIMessage = "option name Spinner type spin default 2 min 0"
        .parse()
        .unwrap();
    assert_eq!(
        msg,
        UCIMessage::Option(UCIOptionConfig::Spin {
            name: "Spinner".to_string(),
            default: Some(2),
            min: Some(0),
            max: None,
        })
    );
}

#[test]
fn parse_option_spin_max_only() {
    let msg: UCIMessage = "option name Spinner type spin default 2 max 4"
        .parse()
        .unwrap();
    assert_eq!(
        msg,
        UCIMessage::Option(UCIOptionConfig::Spin {
            name: "Spinner".to_string(),
            default: Some(2),
            min: None,
            max: Some(4),
        })
    );
}

#[test]
fn parse_option_spin_full() {
    let msg: UCIMessage = "option name Spinner type spin default 2 min 0 max 4"
        .parse()
        .unwrap();
    assert_eq!(
        msg,
        UCIMessage::Option(UCIOptionConfig::Spin {
            name: "Spinner".to_string(),
            default: Some(2),
            min: Some(0),
            max: Some(4),
        })
    );
}

#[test]
fn parse_option_combo() {
    let msg: UCIMessage = "option name combobox type combo default my default option var option 1 var option 2  var my default option"
        .parse()
        .unwrap();
    assert_eq!(
        msg,
        UCIMessage::Option(UCIOptionConfig::Combo {
            name: "combobox".to_string(),
            default: Some("my default option".to_string()),
            var: vec![
                "option 1".to_string(),
                "option 2".to_string(),
                "my default option".to_string(),
            ]
        })
    );
}

#[test]
fn parse_option_combo_empty_string() {
    let msg: UCIMessage =
        "option name combobox type combo default <empty> var option 1 var var <empty>"
            .parse()
            .unwrap();
    assert_eq!(
        msg,
        UCIMessage::Option(UCIOptionConfig::Combo {
            name: "combobox".to_string(),
            default: Some("".to_string()),
            var: vec!["option 1".to_string(), "".to_string(), "".to_string(),]
        })
    );
}

#[test]
fn parse_option_button() {
    let msg: UCIMessage = "option name Push me type button".parse().unwrap();
    assert_eq!(
        msg,
        UCIMessage::Option(UCIOptionConfig::Button {
            name: "Push me".to_string(),
        })
    );
}

#[test]
fn parse_option_string() {
    let msg: UCIMessage =
        "option name Freeform string type string default this is the string default"
            .parse()
            .unwrap();
    assert_eq!(
        msg,
        UCIMessage::Option(UCIOptionConfig::String {
            name: "Freeform string".to_string(),
            default: Some("this is the string default".to_string())
        })
    );
}

#[test]
fn parse_option_string_empty() {
    let msg: UCIMessage = "option name Freeform string type string default <empty>"
        .parse()
        .unwrap();
    assert_eq!(
        msg,
        UCIMessage::Option(UCIOptionConfig::String {
            name: "Freeform string".to_string(),
            default: Some("".to_string())
        })
    );
}
