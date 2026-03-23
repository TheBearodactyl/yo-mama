#[macro_export]
macro_rules! easter_eggs {

    (@arms $joke:ident, $effects:ident, $cx:ident, $cy:ident,) => {};

    (@arms $joke:ident, $effects:ident, $cx:ident, $cy:ident,
        $segment:ident . $field:ident == $constant:literal [$phase:ident] =>
            $effect_name:ident $(($($effect_args:tt)*))? ;
        $($rest:tt)*
    ) => {
        if $joke.$field.hash == $constant {
            $effects.push((
                Segment::$segment,
                Phase::$phase,
                easter_eggs!(@effect $cx, $cy, $effect_name $(($($effect_args)*))?),
            ));
        }
        easter_eggs!(@arms $joke, $effects, $cx, $cy, $($rest)*);
    };

    (@arms $joke:ident, $effects:ident, $cx:ident, $cy:ident,
        $segment:ident . $field:ident == $constant:literal =>
            $effect_name:ident $(($($effect_args:tt)*))? ;
        $($rest:tt)*
    ) => {
        if $joke.$field.hash == $constant {
            $effects.push((
                Segment::$segment,
                Phase::During,
                easter_eggs!(@effect $cx, $cy, $effect_name $(($($effect_args)*))?),
            ));
        }
        easter_eggs!(@arms $joke, $effects, $cx, $cy, $($rest)*);
    };

    (@arms $joke:ident, $effects:ident, $cx:ident, $cy:ident,
        $segment:ident . $field:ident == $constant:path [$phase:ident] =>
            $effect_name:ident $(($($effect_args:tt)*))? ;
        $($rest:tt)*
    ) => {
        if $joke.$field.hash == $constant.hash {
            $effects.push((
                Segment::$segment,
                Phase::$phase,
                easter_eggs!(@effect $cx, $cy, $effect_name $(($($effect_args)*))?),
            ));
        }
        easter_eggs!(@arms $joke, $effects, $cx, $cy, $($rest)*);
    };

    (@arms $joke:ident, $effects:ident, $cx:ident, $cy:ident,
        $segment:ident . $field:ident == $constant:path =>
            $effect_name:ident $(($($effect_args:tt)*))? ;
        $($rest:tt)*
    ) => {
        if $joke.$field.hash == $constant.hash {
            $effects.push((
                Segment::$segment,
                Phase::During,
                easter_eggs!(@effect $cx, $cy, $effect_name $(($($effect_args)*))?),
            ));
        }
        easter_eggs!(@arms $joke, $effects, $cx, $cy, $($rest)*);
    };

    (@effect $cx:ident, $cy:ident, overlay($sprite:expr, scale: $scale:expr)) => {
        EffectAction::Overlay {
            sprite: $sprite, x: $cx, y: $cy, alpha: 255, scale: Some($scale),
        }
    };

    (@effect $cx:ident, $cy:ident, overlay($sprite:expr)) => {
        EffectAction::Overlay {
            sprite: $sprite, x: $cx, y: $cy, alpha: 255, scale: None,
        }
    };

    (@effect $cx:ident, $cy:ident, animated_overlay($sprite:expr, scale: $scale:expr)) => {
        EffectAction::AnimatedOverlay {
            sprite: $sprite, x: $cx, y: $cy, alpha: 255, scale: Some($scale),
        }
    };

    (@effect $cx:ident, $cy:ident, animated_overlay($sprite:expr)) => {
        EffectAction::AnimatedOverlay {
            sprite: $sprite, x: $cx, y: $cy, alpha: 255, scale: None,
        }
    };

    (@effect $cx:ident, $cy:ident, audio($audio:expr, volume: $vol:expr)) => {
        EffectAction::PlayAudio { audio: $audio, volume: $vol }
    };

    (@effect $cx:ident, $cy:ident, crash) => {
        EffectAction::Crash
    };

    ($joke:ident, $center:expr => { $($body:tt)* }) => {{
        let (center_x, center_y) = $center;
        let mut effects = Vec::new();
        easter_eggs!(@arms $joke, effects, center_x, center_y, $($body)*);
        effects
    }};
}
