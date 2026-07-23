//! UEB Appendix 1 longer-word Shortforms List.
//!
//! Source of truth: RUEB 2024 Appendix 1, lines 8768-9065 in the extracted
//! project reference.  These are data definitions from the rulebook, not test
//! fixtures.  The 10.9.3 generative families are deliberately handled by
//! algorithm in `rule_10_9`; this table holds only Appendix-1 words explicitly
//! listed under the base shortforms.

use phf::{Set, phf_set};

pub static APPENDIX_LONGER_WORDS: Set<&'static str> = phf_set! {
    "aboutface", "aboutfaced", "aboutfacer", "aboutfacing", "aboutturn", "aboutturned",
    "eastabout", "gadabout", "hereabout", "knockabout", "layabout", "northabout",
    "rightabout", "roundabout", "roustabout", "runabout", "southabout", "stirabout",
    "thereabout", "turnabout", "walkabout", "westabout", "whereabout",
    "aboveboard", "aboveground", "abovementioned", "hereinabove",
    "accordingly", "unaccording", "unaccordingly", "readacross",
    "afterbattle", "afterbirth", "afterbreakfast", "afterburn", "afterburned",
    "afterburner", "afterburning", "aftercare", "afterclap", "aftercoffee", "afterdamp",
    "afterdark", "afterdeck", "afterdinner", "afterflow", "aftergame", "afterglow",
    "afterguard", "afterhatch", "afterhatches", "afterhour", "afterlife", "afterlight",
    "afterlives", "afterlunch", "afterlunches", "aftermarket", "aftermatch", "aftermatches",
    "aftermath", "aftermeeting", "aftermentioned", "aftermidday", "aftermidnight",
    "aftermost", "afterpain", "afterparties", "afterparty", "afterpiece", "afterplay",
    "aftersale", "afterschool", "aftersensation", "aftershave", "aftershock", "aftershow",
    "aftershower", "aftersupper", "aftertaste", "aftertax", "aftertaxes", "aftertea",
    "aftertheater", "aftertheatre", "afterthought", "aftertime", "aftertreatment",
    "afterword", "afterwork", "afterworld", "hereafter", "hereinafter", "morningafter",
    "thereafter", "thereinafter", "whereafter", "whereinafter",
    "afternoontea", "goodafternoon", "midafternoon",
    "hereagain", "hereinagain", "thereagain", "thereinagain", "whereagain", "whereinagain",
    "hereagainst", "thereagainst", "whereagainst",
    "beforehand", "beforementioned", "behindhand", "belowdeck", "belowground",
    "belowmentioned", "beneathdeck", "beneathground", "betweendeck", "betweentime",
    "betweenwhile", "colorblind", "colorblindness", "colorblindnesses", "colourblind",
    "colourblindness", "colourblindnesses", "deafblind", "deafblindness", "deafblindnesses",
    "purblind", "purblindly", "purblindness", "purblindnesses", "snowblind",
    "snowblindness", "snowblindnesses", "unblindfold", "unblindfolded", "unblindfolding",
    "children'swear", "conceived", "conceiver", "coulda", "couldest", "couldn't",
    "couldn't've", "couldst", "could've", "archdeceiver", "deceived", "deceiver",
    "undeceive", "undeceived", "undeceiver", "undeceiving", "declared", "declarer",
    "undeclare", "undeclared", "feetfirst", "firstaid", "firstaider", "headfirst",
    "tailfirst", "befriend", "boyfriend", "defriend", "galfriend", "gentlemanfriend",
    "gentlemenfriends", "girlfriend", "guyfriend", "ladyfriend", "manfriend", "menfriends",
    "penfriend", "schoolfriend", "unfriend", "unfriendlier", "unfriendliest",
    "unfriendliness", "unfriendlinesses", "unfriendly", "womanfriend", "womenfriends",
    "feelgood", "gooder", "goodest", "goodevening", "goodie", "goodish",
    "goodun", "goody", "goodyear", "scattergood", "supergood", "himbo", "himboes",
    "immediately", "immediateness", "bloodletter", "chainletter", "hateletter",
    "lettered", "letterer", "lettering", "letteropener", "loveletter", "newsletter",
    "reletter", "relettered", "relettering", "unlettered", "belittle", "belittled",
    "belittlement", "belittler", "forasmuch", "inasmuch", "insomuch", "muchly",
    "muchness", "overmuch", "musta", "mustard", "mustardy", "mustier", "mustiest",
    "mustily", "mustiness", "mustn't", "mustn't've", "must've", "musty", "unnecessary",
    "highlypaid", "illpaid", "lowlypaid", "overpaid", "poorlypaid", "postpaid", "prepaid",
    "repaid", "underpaid", "unpaid", "unrepaid", "wellpaid", "apperceive", "apperceived",
    "apperceiver", "misperceive", "misperceived", "misperceiver", "perceived", "perceiver",
    "unperceive", "unperceived", "apperceiving", "misperceiving", "unperceiving",
    "perhapses", "doublequick", "quicken", "quickened", "quickener", "quickening",
    "quicker", "quickest", "quickie", "quickish", "quickishly", "quicky", "superquick",
    "unquick", "preceive", "preceiver", "received", "receiver", "receivership", "unreceived",
    "preceiving", "rejoiced", "rejoiceful", "rejoicefully", "rejoicefulness", "rejoicer",
    "unrejoice", "unrejoiced", "unrejoiceful", "unrejoicefully", "unrejoicefulness",
    "unrejoicer", "rejoicingly", "unrejoicing", "unrejoicingly", "aforesaid", "foresaid",
    "gainsaid", "missaid", "saidest", "saidst", "unsaid", "shoulda", "shouldest",
    "shouldn't", "shouldn't've", "shouldst", "should've", "nonesuch", "nonsuch",
    "somesuch", "suchlike", "togetherness", "'twould", "'twoulda", "'twouldn't",
    "'twouldn't've", "'twould've", "woulda", "wouldest", "wouldn't", "wouldn't've",
    "wouldst", "would've", "do-it-yourselfer",
};

static APPENDIX_MIXED_CASE_LONGER_WORDS: Set<&'static str> = phf_set! {
    // RUEB 2024 Appendix 1, extracted lines 8877-8880, explicitly lists
    // `DeafBlind`; lower-case `penfriend` alone (lines 8923-8928) does not license
    // the interior-capitals form `PenFriend` under §10.9.4.
    "deafblind",
};

pub fn listed_or_added_s(word: &str) -> bool {
    if APPENDIX_LONGER_WORDS.contains(word) {
        return true;
    }
    if matches!(word, "abouts" | "almosts" | "hims") {
        return false;
    }
    match word.strip_suffix('s') {
        Some(base) => APPENDIX_LONGER_WORDS.contains(base),
        None => false,
    }
}

pub fn mixed_case_listed(word: &str) -> bool {
    APPENDIX_MIXED_CASE_LONGER_WORDS.contains(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::listed_base("afterburn", true)]
    #[case::listed_plural("afterburns", true)]
    #[case::listed_longer_variant("afterburner", true)]
    #[case::blocked_abouts("abouts", false)]
    #[case::unlisted_plural("zzzzs", false)]
    #[case::unlisted("zzzz", false)]
    fn listed_or_added_s_paths(#[case] word: &str, #[case] expected: bool) {
        assert_eq!(listed_or_added_s(word), expected);
    }

    #[rstest::rstest]
    #[case::listed("deafblind", true)]
    #[case::unlisted("penfriend", false)]
    fn mixed_case_listed_paths(#[case] word: &str, #[case] expected: bool) {
        assert_eq!(mixed_case_listed(word), expected);
    }
}
