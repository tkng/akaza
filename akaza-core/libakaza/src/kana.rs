use std::collections::HashMap;

pub struct KanaConverter {
    hira2kata_map: HashMap<char, char>,
}

impl Default for KanaConverter {
    fn default() -> Self {
        KanaConverter {
            hira2kata_map: Self::generate_map(),
        }
    }
}

impl KanaConverter {
    fn generate_map() -> HashMap<char, char> {
        HashMap::from([
            ('ぁ', 'ァ'),
            ('あ', 'ア'),
            ('ぃ', 'ィ'),
            ('い', 'イ'),
            ('ぅ', 'ゥ'),
            ('う', 'ウ'),
            ('ぇ', 'ェ'),
            ('え', 'エ'),
            ('ぉ', 'ォ'),
            ('お', 'オ'),
            ('か', 'カ'),
            ('が', 'ガ'),
            ('き', 'キ'),
            ('ぎ', 'ギ'),
            ('く', 'ク'),
            ('ぐ', 'グ'),
            ('け', 'ケ'),
            ('げ', 'ゲ'),
            ('こ', 'コ'),
            ('ご', 'ゴ'),
            ('さ', 'サ'),
            ('ざ', 'ザ'),
            ('し', 'シ'),
            ('じ', 'ジ'),
            ('す', 'ス'),
            ('ず', 'ズ'),
            ('せ', 'セ'),
            ('ぜ', 'ゼ'),
            ('そ', 'ソ'),
            ('ぞ', 'ゾ'),
            ('た', 'タ'),
            ('だ', 'ダ'),
            ('ち', 'チ'),
            ('ぢ', 'ヂ'),
            ('っ', 'ッ'),
            ('つ', 'ツ'),
            ('づ', 'ヅ'),
            ('て', 'テ'),
            ('で', 'デ'),
            ('と', 'ト'),
            ('ど', 'ド'),
            ('な', 'ナ'),
            ('に', 'ニ'),
            ('ぬ', 'ヌ'),
            ('ね', 'ネ'),
            ('の', 'ノ'),
            ('は', 'ハ'),
            ('ば', 'バ'),
            ('ぱ', 'パ'),
            ('ひ', 'ヒ'),
            ('び', 'ビ'),
            ('ぴ', 'ピ'),
            ('ふ', 'フ'),
            ('ぶ', 'ブ'),
            ('ぷ', 'プ'),
            ('へ', 'ヘ'),
            ('べ', 'ベ'),
            ('ぺ', 'ペ'),
            ('ほ', 'ホ'),
            ('ぼ', 'ボ'),
            ('ぽ', 'ポ'),
            ('ま', 'マ'),
            ('み', 'ミ'),
            ('む', 'ム'),
            ('め', 'メ'),
            ('も', 'モ'),
            ('ゃ', 'ャ'),
            ('や', 'ヤ'),
            ('ゅ', 'ュ'),
            ('ゆ', 'ユ'),
            ('ょ', 'ョ'),
            ('よ', 'ヨ'),
            ('ら', 'ラ'),
            ('り', 'リ'),
            ('る', 'ル'),
            ('れ', 'レ'),
            ('ろ', 'ロ'),
            ('わ', 'ワ'),
            ('を', 'ヲ'),
            ('ん', 'ン'),
            ('ー', 'ー'),
            ('ゎ', 'ヮ'),
            ('ゐ', 'ヰ'),
            ('ゑ', 'ヱ'),
            ('ゕ', 'ヵ'),
            ('ゖ', 'ヶ'),
            ('ゔ', 'ヴ'),
            ('ゝ', 'ヽ'),
            ('ゞ', 'ヾ'),
            ('・', '・'),
            ('「', '「'),
            ('」', '」'),
            ('。', '。'),
            ('、', '、'),
        ])
    }

    pub fn hira2kata(&self, src: &str) -> String {
        let mut buf: String = String::new();
        for c in src.chars() {
            buf.push(match self.hira2kata_map.get(&c) {
                Some(kata) => *kata,
                None => c,
            })
        }
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hira2kata() {
        let converter = KanaConverter::default();
        assert_eq!(converter.hira2kata("いうお"), "イウオ".to_string())
    }
}
