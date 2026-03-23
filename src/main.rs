use {rand::seq::IndexedRandom, segments::*};

mod eastereggs;
mod lmao;
mod macros;
mod record;
mod segments;

#[derive(Debug, Clone, Copy)]
pub struct SegmentClip {
    pub data: &'static [u8],
    pub hash: u64,
    pub name: &'static str,
}

impl PartialEq for SegmentClip {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for SegmentClip {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Segment {
    Base,
    Adjective,
    Clause,
    Action,
    Reaction,
}

impl Segment {
    pub const ALL: &[Segment] = &[
        Self::Base,
        Self::Adjective,
        Self::Clause,
        Self::Action,
        Self::Reaction,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Base => "Base",
            Self::Adjective => "Adjective",
            Self::Clause => "Clause",
            Self::Action => "Action",
            Self::Reaction => "Reaction",
        }
    }

    pub fn field_name(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::Adjective => "adjective",
            Self::Clause => "clause",
            Self::Action => "action",
            Self::Reaction => "reaction",
        }
    }

    pub fn all_clips(self) -> &'static [&'static SegmentClip] {
        match self {
            Self::Base => base::ALL,
            Self::Adjective => adjective::ALL,
            Self::Clause => clause::ALL,
            Self::Action => action::ALL,
            Self::Reaction => reaction::ALL,
        }
    }
}

impl std::fmt::Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Before,
    During,
    After,
}

#[derive(Clone, Copy)]
pub enum EffectAction {
    Overlay {
        sprite: &'static [u8],
        x: i32,
        y: i32,
        alpha: u8,
        scale: Option<f32>,
    },

    AnimatedOverlay {
        sprite: &'static [u8],
        x: i32,
        y: i32,
        alpha: u8,
        scale: Option<f32>,
    },

    PlayAudio {
        audio: &'static [u8],
        volume: f32,
    },

    Crash,
}

#[derive(Clone, Copy, Debug)]
pub struct YoMamaJoke {
    pub base: &'static SegmentClip,
    pub adjective: &'static SegmentClip,
    pub clause: &'static SegmentClip,
    pub action: &'static SegmentClip,
    pub reaction: &'static SegmentClip,
}

impl YoMamaJoke {
    pub fn generate() -> Self {
        let mut rng = rand::rng();

        Self {
            base: base::ALL.choose(&mut rng).copied().unwrap(),
            adjective: adjective::ALL.choose(&mut rng).copied().unwrap(),
            clause: clause::ALL.choose(&mut rng).copied().unwrap(),
            action: action::ALL.choose(&mut rng).copied().unwrap(),
            reaction: reaction::ALL.choose(&mut rng).copied().unwrap(),
        }
    }
}

pub fn play_audio(audio: &[u8], volume: f32) {
    let mut handle =
        rodio::DeviceSinkBuilder::open_default_sink().expect("Failed to open default sink");
    handle.log_on_drop(false);
    let player = rodio::Player::connect_new(handle.mixer());
    let seg = std::io::BufReader::new(std::io::Cursor::new(audio.to_vec()));
    let source = rodio::Decoder::new_mp3(seg).expect("Failed to decode segment");

    player.append(source);
    player.set_volume(volume);
    player.play();
    player.sleep_until_end();
}

fn run_phase_effects(effects: &[(Segment, Phase, EffectAction)], segment: Segment, phase: Phase) {
    let matching: Vec<_> = effects
        .iter()
        .filter(|(seg, ph, _)| *seg == segment && *ph == phase)
        .map(|(_, _, action)| *action)
        .collect();

    if matching.is_empty() {
        return;
    }

    let _guards: Vec<_> = matching
        .iter()
        .filter_map(|action| {
            if let EffectAction::Overlay {
                sprite,
                x: _,
                y: _,
                alpha,
                scale,
            } = action
            {
                let x = &0;
                let y = &0;
                lmao::TransparentOverlay::new(sprite, *x, *y, *alpha, *scale).ok()
            } else {
                None
            }
        })
        .collect();

    let _anim_guards: Vec<_> = matching
        .iter()
        .filter_map(|action| {
            if let EffectAction::AnimatedOverlay {
                sprite,
                x: _,
                y: _,
                alpha,
                scale,
            } = action
            {
                let x = &0;
                let y = &0;
                lmao::AnimatedOverlay::new(sprite, *x, *y, *alpha, *scale).ok()
            } else {
                None
            }
        })
        .collect();

    for action in &matching {
        match action {
            EffectAction::PlayAudio { audio, volume } => play_audio(audio, *volume),
            EffectAction::Crash => lmao::crash(),
            EffectAction::Overlay { .. } | EffectAction::AnimatedOverlay { .. } => {}
        }
    }
}

fn collect_during_guards(
    effects: &[(Segment, Phase, EffectAction)],
    segment: Segment,
) -> (Vec<lmao::TransparentOverlay>, Vec<lmao::AnimatedOverlay>) {
    let during: Vec<_> = effects
        .iter()
        .filter(|(seg, ph, _)| *seg == segment && *ph == Phase::During)
        .collect();

    let static_overlays = during
        .iter()
        .filter_map(|(_, _, action)| {
            if let EffectAction::Overlay {
                sprite,
                x: _,
                y: _,
                alpha,
                scale,
            } = action
            {
                let x = &0;
                let y = &0;
                lmao::TransparentOverlay::new(sprite, *x, *y, *alpha, *scale).ok()
            } else {
                None
            }
        })
        .collect();

    let animated_overlays = during
        .iter()
        .filter_map(|(_, _, action)| {
            if let EffectAction::AnimatedOverlay {
                sprite,
                x: _,
                y: _,
                alpha,
                scale,
            } = action
            {
                let x = &0;
                let y = &0;
                lmao::AnimatedOverlay::new(sprite, *x, *y, *alpha, *scale).ok()
            } else {
                None
            }
        })
        .collect();

    (static_overlays, animated_overlays)
}

pub struct YoMama;

impl YoMama {
    fn play() {
        let joke = YoMamaJoke::generate();
        Self::play_joke(joke);
    }

    pub fn play_joke(joke: YoMamaJoke) {
        let effects = eastereggs::register(&joke);

        let steps: &[(Segment, &[u8])] = &[
            (Segment::Base, joke.base.data),
            (Segment::Adjective, joke.adjective.data),
            (Segment::Clause, joke.clause.data),
            (Segment::Action, joke.action.data),
            (Segment::Reaction, joke.reaction.data),
        ];

        for &(segment, audio) in steps {
            run_phase_effects(&effects, segment, Phase::Before);

            let (_guards, _anim_guards) = collect_during_guards(&effects, segment);
            play_audio(audio, 5.0);
            drop(_anim_guards);
            drop(_guards);

            run_phase_effects(&effects, segment, Phase::After);
        }
    }

    #[allow(dead_code)]
    fn play_custom(joke: YoMamaJoke) {
        play_audio(joke.base.data, 5.0);
        play_audio(joke.adjective.data, 5.0);
        play_audio(joke.clause.data, 5.0);
        play_audio(joke.action.data, 5.0);
        play_audio(joke.reaction.data, 5.0);
    }
}

fn main() {
    if cfg!(debug_assertions) {
        debug_menu();
    } else {
        YoMama::play();
    }
}

fn debug_menu() {
    use polyprompt::{Backend, PolyOption, PolyPrompt};

    let prompt = PolyPrompt::new(Backend::Bearask);

    #[derive(Clone, PartialEq, Eq)]
    enum Action {
        PlayRandom,
        PlaySpecific,
        PlaySegment,
        InspectJoke,
        Record,
        ListSegments,
        RegisterEasterEgg,
        DownloadOverlay,
        ListOverlays,
        PreviewOverlay,
        StressTest,
        Exit,
    }

    loop {
        let choice = prompt
            .select::<Action>("Yo Mama Joke Generator - Debug Menu")
            .with_options(vec![
                PolyOption::new("Play a random joke", Action::PlayRandom),
                PolyOption::new("Play a specific joke (pick segments)", Action::PlaySpecific),
                PolyOption::new("Play a single segment", Action::PlaySegment),
                PolyOption::new(
                    "Inspect random joke (segments & easter eggs)",
                    Action::InspectJoke,
                ),
                PolyOption::new("Record a new segment", Action::Record),
                PolyOption::new("List all segments (names & hashes)", Action::ListSegments),
                PolyOption::new("Register an easter egg", Action::RegisterEasterEgg),
                PolyOption::new(
                    "Download an overlay image from URL",
                    Action::DownloadOverlay,
                ),
                PolyOption::new("List overlay images", Action::ListOverlays),
                PolyOption::new("Preview an overlay image", Action::PreviewOverlay),
                PolyOption::new("Stress test (play N random jokes)", Action::StressTest),
                PolyOption::new("Exit", Action::Exit),
            ])
            .run()
            .expect("Failed to get menu selection");

        let result: color_eyre::Result<()> = match choice {
            Action::PlayRandom => {
                YoMama::play();
                Ok(())
            }
            Action::PlaySpecific => record::play_specific_joke(),
            Action::PlaySegment => record::play_single_segment(),
            Action::InspectJoke => {
                record::inspect_random_joke();
                Ok(())
            }
            Action::Record => YoMama::record_new(),
            Action::ListSegments => {
                record::list_segments();
                Ok(())
            }
            Action::RegisterEasterEgg => record::register_easter_egg(),
            Action::DownloadOverlay => record::download_overlay(),
            Action::ListOverlays => {
                record::list_overlays();
                Ok(())
            }
            Action::PreviewOverlay => record::preview_overlay(),
            Action::StressTest => record::stress_test(),
            Action::Exit => break,
        };

        if let Err(e) = result {
            eprintln!("Error: {e}");
        }
    }
}
