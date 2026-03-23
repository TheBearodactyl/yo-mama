use {
    crate::{
        EffectAction, Phase, Segment, YoMamaJoke, easter_eggs, lmao::get_center_of_screen,
        segments::*,
    },
    std::sync::OnceLock,
};

static MONITOR_CENTER: OnceLock<(i32, i32)> = OnceLock::new();

pub fn register(joke: &YoMamaJoke) -> Vec<(Segment, Phase, EffectAction)> {
    let center = MONITOR_CENTER.get_or_init(get_center_of_screen);

    easter_eggs!(joke, *center => {
        Reaction.reaction == 0xa418f96c5ece67d8_u64 => overlay(DIMENTIA_SPR);
        Reaction.reaction == 0xfe37dde2a5170ebd_u64 => overlay(FAMILYGUY_SPR);
        Reaction.reaction == 0xa3102f64af1ea4e9_u64 => overlay(DIES_SPR);
        Reaction.reaction == 0x5b9a5c4598e14d89_u64 => overlay(PI_SPR);
        Reaction.reaction == 0x099be077819d1a3a_u64 => animated_overlay(HEARTATTACK_SPR);
        Reaction.reaction == 0x749c85feca15489e_u64 => overlay(CRIES_SPR);
        Reaction.reaction == 0x69a990e91024452a_u64 [After] => crash;
        Reaction.reaction == 0xca6671f9cd189a00_u64 [After] => overlay(OHCOMEON_SPR);
        Reaction.reaction == 0xca6671f9cd189a00_u64 [After] => audio(OHCOMEON, volume: 5.0);
        Reaction.reaction == 0x51063ab13115f078_u64 => overlay(SMASHBROS_SPR, scale: 0.5);

        Adjective.adjective == 0xfcc84f69b1fd0b0d_u64 => animated_overlay(CRACK_SPR, scale: 3.0);
        Adjective.adjective == 0x19eb879d2845e1bf_u64 => overlay(STUPID_SPR);
        Adjective.adjective == 0x002fbb04f43e9019_u64 => overlay(OLD_SPR);
        Adjective.adjective == 0x113dcdb52dba766e_u64 => overlay(ALCHOHOLIC_SPR);
        Adjective.adjective == 0xafe3de4e2c58e6d0_u64 => overlay(FAT_SPR);
        Adjective.adjective == 0x839074efae23634e_u64 => overlay(SHORT_SPR);
        Adjective.adjective == 0xec65241c772fbcfd_u64 => overlay(NERDY_SPR);
        Adjective.adjective == 0xffa6fc0079212536_u64 => overlay(BROKE_SPR);
        Adjective.adjective == 0xa46fd0bd7291485b_u64 => overlay(SKINNY_SPR);
        Adjective.adjective == 0x839074efae23634e_u64 => overlay(SHORT_SPR);
        Adjective.adjective == 0x40ddaacd9432fd5b_u64 => overlay(PALE_SPR);
        Adjective.adjective == 0x4f0c60bafd9e6a3c_u64 => overlay(HYPER_SPR);
        Adjective.adjective == 0x47721cfdea81b07c_u64 => overlay(AMAZING_SPR);
        Adjective.adjective == 0x226f58895f387c8c_u64 => overlay(BLOB_SPR);
        Adjective.adjective == 0x145e47ffa3e0ff03_u64 => overlay(DOWNTOCLOWN_SPR);
        Adjective.adjective == 0xfc7d1e861165c7f7_u64 => overlay(GENERIC_SPR);
        Adjective.adjective == 0xfde0a2b2f4f90960_u64 => overlay(HATEFUL_SPR);
        Adjective.adjective == 0x31fc975915707e4e_u64 => overlay(HORRIBLE_SPR);
        Adjective.adjective == 0xd3fc92634da7f716_u64 => overlay(HOTLADY_SPR);
        Adjective.adjective == 0xc01710cfa99ada31_u64 => overlay(IMMATURE_SPR);
        Adjective.adjective == 0x0bc5292431f31a6c_u64 => overlay(LAXADAISICAL_SPR);
        Adjective.adjective == 0x79ccb220c8dd3773_u64 => overlay(LOUD_SPR);
        Adjective.adjective == 0xf23af0dbe289ef87_u64 => overlay(MEDIOCRE_SPR);
        Adjective.adjective == 0x3256e54ae52229fb_u64 => overlay(RICH_SPR);
        Adjective.adjective == 0x68e0c8adb871a256_u64 => overlay(SIGMA_SPR);
        Adjective.adjective == 0x1c8f6d3416d4f5ec_u64 => overlay(SMART_SPR);
        Adjective.adjective == 0x55862f1a88e91125_u64 => overlay(STRICT_SPR);
        Adjective.adjective == 0x033ca7e477991156_u64 => overlay(TALL_SPR);
        Adjective.adjective == 0x7555a1fbb2fd6671_u64 => overlay(TRASHY_SPR);

        Base.base == 0x88c0238a9d5bf1c2_u64 => overlay(MOTHER1_SPR);

        Action.action == 0x273d49db452e7495_u64 => overlay(BLOWSNOSE_SPR);
        Action.action == 0x3aa2f394c5e7c472_u64 => overlay(DESPICABLEME3_SPR);
        Action.action == 0xd9381b779d0f8f94_u64 => overlay(SCOTTPILGRIM4_SPR);

        Reaction.reaction == 0x7d0901701df48505_u64 => overlay(BLOODYMURDER_SPR);

        Reaction.reaction == 0xeb5520c18bc5534b_u64 => overlay(BLUNT_SPR);

        Reaction.reaction == 0x0d841944a5bd575a_u64 => overlay(BLUNTINCLUSIVE_SPR);

        Reaction.reaction == 0xc961e027683d67c0_u64 => overlay(BRIDGETROLL_SPR);

        Reaction.reaction == 0x47d196e888928f6c_u64 => overlay(FAINTED_SPR);

        Reaction.reaction == 0xecde990e948436e0_u64 => overlay(GODSNUTS_SPR);

        Reaction.reaction == 0x8e54cbc035681c29_u64 => overlay(MAFIABOSS_SPR);

        Reaction.reaction == 0xe9520991637e16f4_u64 => overlay(OFFENDEDLADY_SPR);

        Reaction.reaction == 0x57b12992cf39e8e7_u64 => overlay(SMOKINGCRACKREACTION_SPR);

        Reaction.reaction == 0x81d077a5312fc798_u64 => overlay(VITAMINGUMMIESLADY_SPR);

        Reaction.reaction == 0x52149ced73ac866c_u64 => overlay(ROCKTHROWING_SPR);

        Reaction.reaction == 0x1c534ea0ea13deba_u64 => overlay(WORLDCONSUMPTION_SPR);

        Action.action == 0x46903600f3e6fc91_u64 => overlay(POPCORN_SPR);

        Action.action == 0xd3b79da93d77a7e4_u64 => overlay(GETTINGDRESSED_SPR);

        Action.action == 0xb45194668c1bcca6_u64 => overlay(GROUNDINGYOU_SPR);

        Action.action == 0xf23e64e05d59d0b9_u64 => overlay(JUMPS_SPR);

        Action.action == 0xce0e29edfa9eb4ae_u64 => overlay(MEETSJUGGALO_SPR);

        Action.action == 0x9c50268de0d2ef23_u64 => overlay(MOWINGLAWN_SPR);

        Action.action == 0xc9ffa1b552387a66_u64 => overlay(PETSCAT_SPR);

        Action.action == 0x09881a5900c34c9e_u64 => overlay(PETSDOG_SPR);

        Action.action == 0xc4b3e04ceb2524b0_u64 => overlay(PLAYSBOARDGAMES_SPR);

        Action.action == 0x434abb4c78232e58_u64 => overlay(PLAYSVIDEOGAMES_SPR);

        Action.action == 0x23404bb8f7655009_u64 => overlay(ROCKYHORROR_SPR);

        Action.action == 0xc489e19f10ff4a84_u64 => overlay(SEESCODE_SPR);

        Action.action == 0xae9ea65271944f2c_u64 => overlay(SLEEPING_SPR);

        Action.action == 0x2927ac578469c187_u64 => overlay(TAKINGOUTTRASH_SPR);

        Action.action == 0x37f49c51b4af7cc7_u64 => overlay(TAKESPHOTO_SPR);

        Action.action == 0x95c9e745af61f958_u64 => overlay(TALKING_SPR);

        Action.action == 0xd7dd5e461f75e811_u64 => overlay(WALKING_SPR);

        Action.action == 0x85c444c8d4581c9c_u64 => overlay(SMOKINGCRACKACTION_SPR);

        Base.base == 0xa4e337abdb941a06_u64 => overlay(MOTHER2_SPR);
    })
}
