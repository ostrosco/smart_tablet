use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    // Holds a map for all unique numbers in the given language. This should hold all possible
    // prefixes for a number.
    //
    // TODO: In the future, this should be a file someplace that we load in based on the given
    // language akin to the commands_{LANG}.json file.
    static ref NUMBER_MAP: HashMap<String, i64> = {
        let mut m = HashMap::new();
        m.insert("zero".into(), 0);
        m.insert("one".into(), 1);
        m.insert("two".into(), 2);
        m.insert("three".into(), 3);
        m.insert("four".into(), 4);
        m.insert("five".into(), 5);
        m.insert("six".into(), 6);
        m.insert("seven".into(), 7);
        m.insert("eight".into(), 8);
        m.insert("nine".into(), 9);
        m.insert("ten".into(), 10);
        m.insert("eleven".into(), 11);
        m.insert("twelve".into(), 12);
        m.insert("thirteen".into(), 13);
        m.insert("fourteen".into(), 14);
        m.insert("fifteen".into(), 15);
        m.insert("sixteen".into(), 16);
        m.insert("seventeen".into(), 17);
        m.insert("eighteen".into(), 18);
        m.insert("ninteen".into(), 19);
        m
    };

    // These are numbers that can chain into another number. We need to handle these cases
    // specifically when we're doing numbers that chain in between magnitudes, such as
    // "five hundred thirty six thousand".
    static ref CHAINS_MAP: HashMap<String, i64> = {
        let mut m = HashMap::new();
        m.insert("twenty".into(), 20);
        m.insert("thirty".into(), 30);
        m.insert("fourty".into(), 40);
        m.insert("fifty".into(), 50);
        m.insert("sixty".into(), 60);
        m.insert("seventy".into(), 70);
        m.insert("eighty".into(), 80);
        m.insert("ninety".into(), 90);
        m
    };

    static ref MAGNITUDE_MAP: HashMap<String, i64> = {
        let mut m = HashMap::new();
        m.insert("hundred".into(), 100);
        m.insert("thousand".into(), 1e3 as i64);
        m.insert("million".into(), 1e6 as i64);
        m.insert("billion".into(), 1e9 as i64);
        m
    };

    // A vector of words that can be used to chain numbers together. In English, there's only "and"
    // but I'm not sure if there are other languages that have more than this. Hence, why this is
    // going to be a vector.
    static ref NUMBER_CHAIN_WORDS: Vec<String> = vec!["and".into()];
}

/// Runs through a voice command and translates any numbers said into an actual number. This
/// implementation is likely going to be heavily biases towards English as I'm not familiar with
/// the number structure of every language.
pub fn parse_number_from_voice(command: &str) -> Option<i64> {
    let mut total = None;
    let mut curr_number = None;
    let mut found_prefix = false;
    let mut number_chain = false;

    for word in command.split_whitespace() {
        if let Some(val) = NUMBER_MAP.get(word) {
            // These are numbers that are prefixes but not the start of a chain, i.e. we shouldn't
            // expect two of these types of numbers in a row.
            if !number_chain {
                total = total.or(Some(0)).map(|x| x + curr_number.unwrap_or(0));
                curr_number = Some(*val);
            } else {
                curr_number = curr_number.or(Some(0)).map(|x| x + *val);
            }
            found_prefix = true;
        } else if let Some(val) = CHAINS_MAP.get(word) {
            // This is reserved for numbers that aren't magnitudes but should be treated as chains
            // for the purposes of construction. In English, these are the -ty numbers.
            curr_number = curr_number.or(Some(0)).map(|x| x + *val);
            number_chain = true;
        } else if let Some(val) = MAGNITUDE_MAP.get(word) {
            // In English, we'll often say something like "a thousand" to indicate one thousand. So
            // we need to handle that special case here where a magnitude is used without a
            // preceeding number.
            if found_prefix {
                curr_number = curr_number.or(Some(0)).map(|x| x * *val);
            } else {
                curr_number = Some(*val);
                found_prefix = true;
            }
            number_chain = false;
        } else if NUMBER_CHAIN_WORDS.contains(&word.to_string()) {
            // Since sometimes, numbers are chained together with special words ("and" in English),
            // we handle that case here to know that we're not done constructing this part of the
            // number.
            number_chain = true;
        } else {
            found_prefix = false;
            number_chain = false;
        }
    }

    // Since prefix numbers are their own number themselves, we'll terminate the above loop without
    // adding the last number we parsed to the total.
    if curr_number.is_some() {
        total = total.or(Some(0)).map(|x| x + curr_number.unwrap_or(0));
    }

    total
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_simple_numbers() {
        assert_eq!(parse_number_from_voice("six"), Some(6));
        assert_eq!(parse_number_from_voice("seventeen"), Some(17));
        assert_eq!(parse_number_from_voice("ninety"), Some(90));
    }

    #[test]
    fn parse_combo_numbers() {
        assert_eq!(parse_number_from_voice("seventy six"), Some(76));
        assert_eq!(parse_number_from_voice("fifty two"), Some(52));
        assert_eq!(
            parse_number_from_voice("nine hundred ninety nine"),
            Some(999)
        );
    }

    #[test]
    fn parse_no_prefix() {
        assert_eq!(parse_number_from_voice("a hundred thousand"), Some(100_000));
        assert_eq!(
            parse_number_from_voice("a million and six"),
            Some(1e6 as i64 + 6)
        );
    }

    #[test]
    fn parse_num_with_mag() {
        assert_eq!(parse_number_from_voice("five hundred"), Some(500));
        assert_eq!(
            parse_number_from_voice("five hundred and thirty"),
            Some(530)
        );
        assert_eq!(
            parse_number_from_voice("five hundred thirty six"),
            Some(536)
        );
    }

    #[test]
    fn parse_hundred_thousand() {
        // Test with our chain word.
        assert_eq!(
            parse_number_from_voice(
                "one million five hundred and thirty six thousand one hundred and two"
            ),
            Some(1_536_102)
        );

        // Test without our chain word.
        assert_eq!(
            parse_number_from_voice(
                "one million five hundred thirty six thousand one hundred and two"
            ),
            Some(1_536_102)
        );

        assert_eq!(
            parse_number_from_voice("five hundred thirty thousand"),
            Some(530_000)
        );
    }

    #[test]
    fn parse_with_chains() {
        assert_eq!(
            parse_number_from_voice("one hundred million and five"),
            Some(1e8 as i64 + 5)
        );
    }
}
