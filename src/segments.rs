include!(concat!(env!("OUT_DIR"), "/segments.rs"));

macro_rules! include_file {
    ($id:ident => $path:expr) => {
        pub static $id: &[u8] = include_bytes!($path);
    };

    ($($id:ident => $path:expr)*) => {
        $(
            pub static $id: &[u8] = include_bytes!($path);
        )*
    };
}

include_file! {
    FAMILYGUY_SPR => "../assets/overlays/familyguy.jpg"
    CRACK_SPR => "../assets/overlays/crack.gif"
    OHCOMEON_SPR => "../assets/overlays/shes17scott.jpg"
    OHCOMEON => "../assets/OHCOMEON.mp3"
    DIMENTIA_SPR => "../assets/overlays/dimentia.jpg"
    DIES_SPR => "../assets/overlays/dies.jpg"
    PI_SPR => "../assets/overlays/pi.jpg"
    CRIES_SPR => "../assets/overlays/cries.jpg"
    HEARTATTACK_SPR => "../assets/overlays/heartattack.gif"
    OLD_SPR => "../assets/overlays/old.jpg"
    STUPID_SPR => "../assets/overlays/stupid.jpg"
    ALCHOHOLIC_SPR => "../assets/overlays/alchoholic.jpg"
    FAT_SPR => "../assets/overlays/fat.jpg"
    MOTHER1_SPR => "../assets/overlays/mother1.jpg"
    NERDY_SPR => "../assets/overlays/nerdy.png"
    BROKE_SPR => "../assets/overlays/broke.jpg"
    SKINNY_SPR => "../assets/overlays/skinny.png"
    SHORT_SPR => "../assets/overlays/short.jpg"
    PALE_SPR => "../assets/overlays/pale.jpg"
    BLOWSNOSE_SPR => "../assets/overlays/blowsnose.jpg"
    DESPICABLEME3_SPR => "../assets/overlays/despicableme3.jpg"
    SCOTTPILGRIM4_SPR => "../assets/overlays/scottpilgrim4.jpg"
    HYPER_SPR => "../assets/overlays/hyper.jpg"
    AMAZING_SPR => "../assets/overlays/amazing.jpg"
    BLOB_SPR => "../assets/overlays/blob.png"
    SMASHBROS_SPR => "../assets/overlays/smashbros.png"
    DOWNTOCLOWN_SPR => "../assets/overlays/downtoclown.jpg"
    GENERIC_SPR => "../assets/overlays/generic.png"
    HATEFUL_SPR => "../assets/overlays/hateful.jpg"
    HORRIBLE_SPR => "../assets/overlays/horrible.jpg"
    HOTLADY_SPR => "../assets/overlays/hotlady.png"
    IMMATURE_SPR => "../assets/overlays/immature.png"
    LAXADAISICAL_SPR => "../assets/overlays/laxadaisical.png"
    LOUD_SPR => "../assets/overlays/loud.jpg"
    MEDIOCRE_SPR => "../assets/overlays/mediocre.jpg"
    RICH_SPR => "../assets/overlays/rich.png"
    SIGMA_SPR => "../assets/overlays/sigma.jpg"
    SMART_SPR => "../assets/overlays/smart.jpg"
    STRICT_SPR => "../assets/overlays/strict.jpg"
    TALL_SPR => "../assets/overlays/tall.jpg"
    TRASHY_SPR => "../assets/overlays/trashy.png"
}
