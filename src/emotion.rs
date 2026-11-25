use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Emotion {
    Happy,
    Sad,
    Excited,
    Angry,
    Sleepy,
    Lonely,
    Proud,
    Concerned,
}

impl Emotion {
    pub fn label(&self) -> &'static str {
        match self {
            Emotion::Happy => "Happy",
            Emotion::Sad => "Sad",
            Emotion::Excited => "Excited",
            Emotion::Angry => "Angry",
            Emotion::Sleepy => "Sleepy",
            Emotion::Lonely => "Lonely",
            Emotion::Proud => "Proud",
            Emotion::Concerned => "Concerned",
        }
    }

    pub fn animated_sprite(
        &self,
        level: u32,
        frame: u64,
        impact_frames: u8,
        vy: f64,
        vx: f64,
    ) -> &'static str {
        let phase = (frame / 8) % 4;

        if impact_frames > 0 {
            return match phase % 2 {
                0 => "  *bonk*  \n (X_X) ;  ",
                _ => "  !oof!   \n  (>_<)   ",
            };
        }

        if vy < -4.0 {
            return match level {
                50.. => "  ✦  ^  ✦  \n \\(≧▽≦)/ ",
                10.. => "   ^  ^   \n  (≧▽≦)/  ",
                _ => "    ^     \n  (^o^)/   ",
            };
        }

        let walking = vx.abs() > 1.5;
        match self {
            Emotion::Happy => {
                if walking {
                    match phase % 2 {
                        0 => "  .·.  \n (^_^) \n  / \\  ",
                        _ => "  ·.·  \n (-_-) \n  | |  ",
                    }
                } else {
                    match phase {
                        0 => "   .   \n  (^_^)  ",
                        1 => "   ·   \n  (-_-)  ",
                        2 => "   ✦   \n  (^_^)  ",
                        _ => "   ·   \n  (-_-)  ",
                    }
                }
            }
            Emotion::Excited | Emotion::Proud => match phase % 2 {
                0 => "  ★ . ★  \n  (^o^)/  ",
                _ => "   ✦ · ✦ \n \\(^o^)/ ",
            },
            Emotion::Sad | Emotion::Lonely => match phase % 2 {
                0 => "   ; ;   \n  (T_T)   ",
                _ => "   . .   \n  (╥﹏╥)  ",
            },
            Emotion::Angry => match phase % 2 {
                0 => "  \\   /  \n  (¬_¬)  ",
                _ => "   \\ /   \n  (ಠ_ಠ)  ",
            },
            Emotion::Sleepy => match phase % 2 {
                0 => "      z  \n  (-_-)  ",
                _ => "       Z \n  (-.-)  ",
            },
            Emotion::Concerned => match phase % 2 {
                0 => "    ?    \n  (o_o;)  ",
                _ => "    !    \n  (・_・;) ",
            },
        }
    }

    pub fn face(&self, level: u32, impact_frames: u8, blink: bool) -> &'static str {
        if impact_frames > 0 {
            return "(>_<)";
        }
        if level >= 100 {
            return match self {
                Emotion::Happy => "✧･ﾟ:*(＾▽＾)*:･ﾟ✧",
                Emotion::Sad => "｡･ﾟﾟ*(>д<)*ﾟﾟ･｡",
                Emotion::Excited => "★~(◠‿◕✿)~★",
                Emotion::Angry => "ψ(｀∇´)ψ",
                Emotion::Sleepy => "(－_－) zzZ ✦",
                Emotion::Lonely => "(｡•́︿•̀｡) ｡･ﾟ",
                Emotion::Proud => "＼(★^∀^★)／",
                Emotion::Concerned => "(⊙_⊙;) ✧",
            };
        }
        if level >= 50 {
            return match self {
                Emotion::Happy => "༼ つ ◕_◕ ༽つ",
                Emotion::Sad => "༼ つ ಥ_ಥ ༽つ",
                Emotion::Excited => "＼(＾O＾)／",
                Emotion::Angry => "(ノಠ益ಠ)ノ彡┻━┻",
                Emotion::Sleepy => "༼ つ -_- ༽つ zzZ",
                Emotion::Lonely => "༼ ༎ຶ ෴ ༎ຶ༽",
                Emotion::Proud => "(ง •̀_•́)ง✧",
                Emotion::Concerned => "༼ つ ⊙_⊙ ༽つ",
            };
        }
        if level >= 10 {
            return match self {
                Emotion::Happy => {
                    if blink {
                        "(-‿-)"
                    } else {
                        "(≧▽≦)"
                    }
                }
                Emotion::Sad => "(╥﹏╥)",
                Emotion::Excited => "(*^▽^*)",
                Emotion::Angry => "(ಠ_ಠ)",
                Emotion::Sleepy => "(￣o￣) . z Z",
                Emotion::Lonely => "(；′⌒`)",
                Emotion::Proud => "(•̀ᴗ•́)و ̑̑",
                Emotion::Concerned => "(・_・;)",
            };
        }
        match self {
            Emotion::Happy => {
                if blink {
                    "(-_-)"
                } else {
                    "(^_^)"
                }
            }
            Emotion::Sad => "(T_T)",
            Emotion::Excited => "(^o^)/",
            Emotion::Angry => "(>_<)",
            Emotion::Sleepy => "(-_-) zZ",
            Emotion::Lonely => "(._.)",
            Emotion::Proud => "(^_~)b",
            Emotion::Concerned => "(o_o;)",
        }
    }

    pub fn random_thought(&self) -> &'static str {
        let mut rng = rand::thread_rng();
        let lines: &[&str] = match self {
            Emotion::Happy => &[
                "Life in the terminal is cozy today.",
                "Ready whenever you are!",
                "Doing great! Keep that rhythm going.",
            ],
            Emotion::Sad => &[
                "You forgot a deadline... it hurts a little.",
                "I'll just sit here in the rain...",
                "We can still bounce back. Tell me what's next.",
            ],
            Emotion::Excited => &[
                "Great job!! Look at that momentum!",
                "UNSTOPPABLE! What's our next target?!",
                "Dopamine levels critical! Let's gooo!",
            ],
            Emotion::Angry => &[
                "(¬_¬) Still scrolling instead of shipping?",
                "Those tasks aren't going to complete themselves!",
                "Hey! Focus! I believe in you, now prove it!",
            ],
            Emotion::Sleepy => &[
                "*yawn*... So quiet in here...",
                "Wake me up when we're ready to grind...",
                "Counting CPU cycles... 1... 2... zzz...",
            ],
            Emotion::Lonely => &[
                "Hello? Anyone behind the keyboard?",
                "I miss clicking keys with you...",
                "It's dark in /dev/null when you're away.",
            ],
            Emotion::Proud => &[
                "Look how far you've come. Truly impressive.",
                "Streak maintained! You're built different.",
                "Masterclass execution today.",
            ],
            Emotion::Concerned => &[
                "Heads up! A deadline is getting really close!",
                "Tick tock... should we wrap that task up soon?",
                "I'm getting nervous about the clock...",
            ],
        };
        lines.choose(&mut rng).unwrap_or(&"...")
    }
}
