use crate::board::{Board, Rules, Turn};
use crate::bug::Bug;
use crate::eval::BasicEvaluator;
use crate::mcts::BiasedRollouts;
use crate::notation::engine_version;
use crate::random::Random;
use minimax::{
    IterativeOptions, IterativeSearch, MCTSOptions, MonteCarloTreeSearch, ParallelOptions,
    ParallelSearch, Strategy,
};
use std::time::Duration;

pub(crate) trait Player {
    fn name(&self) -> String;
    fn new_game(&mut self, game_string: &str);

    fn play_move(&mut self, m: Turn);
    fn undo_move(&mut self, m: Turn);
    fn generate_move(&mut self) -> Turn;
    fn principal_variation(&self) -> Vec<Turn> {
        Vec::new()
    }

    fn set_max_depth(&mut self, depth: u8);
    fn set_timeout(&mut self, time: Duration);
}

struct EnginePlayer {
    board: Board,
    strategy: Box<dyn Strategy<Rules>>,
    random_opening: bool,
    name: String,
}
impl EnginePlayer {
    fn new(strategy: Box<dyn Strategy<Rules>>, random_opening: bool) -> Self {
        Self::new_with_name(
            Some(&format!("engine {}", engine_version())),
            strategy,
            random_opening,
        )
    }
    fn new_with_name(
        name: Option<&str>,
        mut strategy: Box<dyn Strategy<Rules>>,
        random_opening: bool,
    ) -> Self {
        strategy.set_timeout(Duration::from_secs(5));
        if let Some(name) = name {
            EnginePlayer {
                board: Board::default(),
                strategy,
                random_opening,
                name: name.to_owned(),
            }
        } else {
            Self::new(strategy, random_opening)
        }
    }
}

impl Player for EnginePlayer {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn new_game(&mut self, game_string: &str) {
        self.board = Board::from_game_string(game_string).unwrap();
    }

    fn play_move(&mut self, m: Turn) {
        self.board.apply(m);
    }
    fn undo_move(&mut self, m: Turn) {
        self.board.undo(m);
    }
    fn generate_move(&mut self) -> Turn {
        if self.random_opening {
            if self.board.turn_num < 2 {
                loop {
                    let mut random_strategy = Random::<Rules>::default();
                    let turn = random_strategy.choose_move(&self.board).unwrap();
                    if let Turn::Place(_, bug) = turn {
                        if matches!(bug, Bug::Ant | Bug::Grasshopper | Bug::Beetle | Bug::Spider) {
                            return turn;
                        }
                    }
                }
            }
        }
        self.strategy.choose_move(&self.board).unwrap()
    }
    fn principal_variation(&self) -> Vec<Turn> {
        self.strategy.principal_variation()
    }
    fn set_max_depth(&mut self, depth: u8) {
        self.strategy.set_max_depth(depth);
    }

    fn set_timeout(&mut self, time: Duration) {
        self.strategy.set_timeout(time);
    }
}

pub(crate) enum PlayerStrategy {
    Iterative(ParallelOptions),
    Random,
    Mcts(MCTSOptions),
}

pub struct PlayerConfig {
    pub(crate) num_threads: Option<usize>,
    pub(crate) opts: IterativeOptions,
    pub(crate) strategy: PlayerStrategy,
    pub(crate) eval: BasicEvaluator,
    pub(crate) random_opening: bool,
    pub(crate) player_name: Option<String>,
}

pub fn configure_players() -> Result<(PlayerConfig, PlayerConfig, Vec<String>), pico_args::Error> {
    let mut args = pico_args::Arguments::from_env();

    let mut config1 = PlayerConfig::new();
    let mut config2 = PlayerConfig::new();
    let player1_name: Option<String> = args.opt_value_from_str("--player1-name")?;
    let player2_name: Option<String> = args.opt_value_from_str("--player2-name")?;
    config1.player_name = player1_name.clone();
    config2.player_name = player2_name.clone();

    // Configure common minimax options.
    if args.contains("--player1-verbose") {
        config1.opts = config1.opts.verbose();
    }
    if args.contains("--player2-verbose") {
        config2.opts = config2.opts.verbose();
    }
    let window_arg: Option<u32> = args.opt_value_from_str("--player1-aspiration-window")?;
    if let Some(window) = window_arg {
        config1.opts = config1
            .opts
            .with_aspiration_window(window as minimax::Evaluation);
    }
    let window_arg: Option<u32> = args.opt_value_from_str("--player2-aspiration-window")?;
    if let Some(window) = window_arg {
        config2.opts = config2
            .opts
            .with_aspiration_window(window as minimax::Evaluation);
    }
    if args.contains("--player1-double-step") {
        config1.opts = config1.opts.with_double_step_increment();
    }
    if args.contains("--player2-double-step") {
        config2.opts = config2.opts.with_double_step_increment();
    }
    if args.contains("--player1-null-move-pruning") {
        config1.opts = config1.opts.with_null_move_depth(3);
    }
    if args.contains("--player2-null-move-pruning") {
        config2.opts = config2.opts.with_null_move_depth(3);
    }
    if args.contains("--player1-quiet-search") {
        config1.opts = config1.opts.with_quiescence_search_depth(2);
    }
    if args.contains("--player2-quiet-search") {
        config2.opts = config2.opts.with_quiescence_search_depth(2);
    }

    // 0 for num_cpu threads; >0 for specific count.
    config1.num_threads =
        args.opt_value_from_str("--player1-num-threads")?
            .map(|thread_arg: String| {
                if thread_arg == "max" || thread_arg == "all" {
                    0
                } else if let Ok(num) = thread_arg.parse::<usize>() {
                    num
                } else {
                    exit(format!(
                        "Could not parse num_threads={thread_arg}. Expected int or 'max'"
                    ));
                }
            });

    config2.num_threads =
        args.opt_value_from_str("--player2-num-threads")?
            .map(|thread_arg: String| {
                if thread_arg == "max" || thread_arg == "all" {
                    0
                } else if let Ok(num) = thread_arg.parse::<usize>() {
                    num
                } else {
                    exit(format!(
                        "Could not parse num_threads={thread_arg}. Expected int or 'max'"
                    ));
                }
            });

    // Configure specific strategy.
    let player1_strategy: Option<String> = args.opt_value_from_str("--player1-strategy")?;
    config1.strategy = match player1_strategy.as_deref().unwrap_or("iterative") {
        "random" => PlayerStrategy::Random,
        "mcts" => {
            let mut options = MCTSOptions::default()
                .with_max_rollout_depth(200)
                .with_rollouts_before_expanding(5);
            options.verbose = config1.opts.verbose;
            PlayerStrategy::Mcts(options)
        }
        "mtdf" => {
            config1.opts = config1.opts.with_mtdf();
            config1.num_threads = Some(1);
            PlayerStrategy::Iterative(ParallelOptions::new())
        }
        "iterative" => {
            let mut parallel_opts = ParallelOptions::new();
            if args.contains("--player1-background-ponder") {
                parallel_opts = parallel_opts.with_background_pondering();
            }
            PlayerStrategy::Iterative(parallel_opts)
        }
        _ => exit(format!(
            "Unrecognized strategy: {}",
            player1_strategy.unwrap_or_default()
        )),
    };

    let player2_strategy: Option<String> = args.opt_value_from_str("--player2-strategy")?;
    config2.strategy = match player2_strategy.as_deref().unwrap_or("iterative") {
        "random" => PlayerStrategy::Random,
        "mcts" => {
            let mut options = MCTSOptions::default()
                .with_max_rollout_depth(200)
                .with_rollouts_before_expanding(5);
            options.verbose = config2.opts.verbose;
            PlayerStrategy::Mcts(options)
        }
        "mtdf" => {
            config2.opts = config2.opts.with_mtdf();
            config2.num_threads = Some(1);
            PlayerStrategy::Iterative(ParallelOptions::new())
        }
        "iterative" => {
            let mut parallel_opts = ParallelOptions::new();
            if args.contains("--player2-background-ponder") {
                parallel_opts = parallel_opts.with_background_pondering();
            }
            PlayerStrategy::Iterative(parallel_opts)
        }
        _ => exit(format!(
            "Unrecognized strategy: {}",
            player2_strategy.unwrap_or_default()
        )),
    };
    Ok((
        config1,
        config2,
        args.finish()
            .into_iter()
            .map(|s| s.into_string().unwrap())
            .collect::<Vec<_>>(),
    ))
}

pub fn configure_player() -> Result<(PlayerConfig, Vec<String>), pico_args::Error> {
    let mut args = pico_args::Arguments::from_env();

    let mut config = PlayerConfig::new();

    // Configure common minimax options.
    if args.contains(["-v", "--verbose"]) {
        config.opts = config.opts.verbose();
    }
    let table_size: Option<usize> = args.opt_value_from_str("--table_mb")?;
    if let Some(table_size) = table_size {
        config.opts.table_byte_size = table_size.checked_shl(20).unwrap();
    }
    let window_arg: Option<u32> = args.opt_value_from_str("--aspiration-window")?;
    if let Some(window) = window_arg {
        config.opts = config
            .opts
            .with_aspiration_window(window as minimax::Evaluation);
    }
    if args.contains("--double-step") {
        config.opts = config.opts.with_double_step_increment();
    }
    if args.contains("--null-move-pruning") {
        config.opts = config.opts.with_null_move_depth(3);
    }
    if args.contains("--quiet-search") {
        config.opts = config.opts.with_quiescence_search_depth(2);
    }

    // 0 for num_cpu threads; >0 for specific count.
    config.num_threads = args
        .opt_value_from_str("--num-threads")?
        .map(|thread_arg: String| {
            if thread_arg == "max" || thread_arg == "all" {
                0
            } else if let Ok(num) = thread_arg.parse::<usize>() {
                num
            } else {
                exit(format!(
                    "Could not parse num_threads={thread_arg}. Expected int or 'max'"
                ));
            }
        });

    // Configure specific strategy.
    let strategy: Option<String> = args.opt_value_from_str("--strategy")?;
    config.strategy = match strategy.as_deref().unwrap_or("iterative") {
        "random" => PlayerStrategy::Random,
        "mcts" => {
            let mut options = MCTSOptions::default()
                .with_max_rollout_depth(200)
                .with_rollouts_before_expanding(5);
            options.verbose = config.opts.verbose;
            PlayerStrategy::Mcts(options)
        }
        "mtdf" => {
            config.opts = config.opts.with_mtdf();
            config.num_threads = Some(1);
            PlayerStrategy::Iterative(ParallelOptions::new())
        }
        "iterative" => {
            let mut parallel_opts = ParallelOptions::new();
            if args.contains("--background-ponder") {
                parallel_opts = parallel_opts.with_background_pondering();
            }
            PlayerStrategy::Iterative(parallel_opts)
        }
        _ => exit(format!(
            "Unrecognized strategy: {}",
            strategy.unwrap_or_default()
        )),
    };
    Ok((
        config,
        args.finish()
            .into_iter()
            .map(|s| s.into_string().unwrap())
            .collect::<Vec<_>>(),
    ))
}

fn exit(msg: String) -> ! {
    eprintln!("{msg}");
    std::process::exit(1)
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerConfig {
    pub fn new() -> Self {
        Self {
            num_threads: None,
            opts: IterativeOptions::new()
                .with_countermoves()
                .with_countermove_history()
                .with_table_byte_size(100 << 20),
            strategy: PlayerStrategy::Iterative(ParallelOptions::new()),
            eval: BasicEvaluator::default(),
            random_opening: false,
            player_name: None,
        }
    }

    pub(crate) fn new_player(&self) -> Box<dyn Player> {
        Box::new(match &self.strategy {
            PlayerStrategy::Random => EnginePlayer::new_with_name(
                self.player_name.as_deref().or(Some("random")),
                Box::<Random<Rules>>::default(),
                self.random_opening,
            ),
            PlayerStrategy::Mcts(opts) => {
                let mut opts = opts.clone();
                let num_threads = self.num_threads.unwrap_or(0);
                if num_threads > 0 {
                    opts = opts.with_num_threads(num_threads);
                }
                EnginePlayer::new_with_name(
                    self.player_name.as_deref(),
                    Box::new(MonteCarloTreeSearch::new_with_policy(
                        opts,
                        Box::new(BiasedRollouts {}),
                    )),
                    self.random_opening,
                )
            }
            PlayerStrategy::Iterative(parallel_opts) => {
                let mut parallel_opts = *parallel_opts;
                let num_threads = self.num_threads.unwrap_or(0);
                if num_threads > 0 {
                    parallel_opts = parallel_opts.with_num_threads(num_threads);
                }
                EnginePlayer::new_with_name(
                    self.player_name.as_deref(),
                    if num_threads == 1 {
                        Box::new(IterativeSearch::new(self.eval, self.opts))
                    } else {
                        Box::new(ParallelSearch::new(self.eval, self.opts, parallel_opts))
                    },
                    self.random_opening,
                )
            }
        })
    }
}
