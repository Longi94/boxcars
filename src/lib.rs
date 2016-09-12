#![allow(dead_code)]
#![allow(unused_imports)]
#![feature(plugin, custom_derive)]
#![plugin(serde_macros)]

#[macro_use]
extern crate nom;
extern crate serde;

use nom::{HexDisplay, Needed, IResult, ErrorKind, le_i32, le_u64, le_u32, le_u8, le_u16, length_value, le_f32,
          FileProducer};
use nom::Err;
use nom::IResult::*;

pub use self::models::*;
mod models;

named!(length_encoded,
       chain!(
        size: le_u32 ~
        crc: le_u32 ~
        data: take!(size),
        || {data}
    ));

/// Text is encoded with a leading int that denotes the number of bytes that
/// the text spans. The last byte in the text will be null terminated, so we trim
/// it off. It may seem redundant to store this information, but stackoverflow contains
/// a nice reasoning for why it may have been done this way:
/// http://stackoverflow.com/q/6293457/433785
named!(text_encoded<&[u8], &str>,
    chain!(
        size: le_u32 ~
        data: take_str!(size - 1) ~
        take!(1),
        || {data}
    )
);

/// Header properties are encoded in a pretty simple format, with some oddities. The first 64bits
/// is data that can be discarded, some people think that the 64bits is the length of the data
/// while others think that the first 32bits is the header length in bytes with the subsequent
/// 32bits unknown. Doesn't matter to us, we throw it out anyways. The rest of the bytes are
/// decoded property type specific.

named!(str_prop<&[u8], HeaderProp>,
  chain!(le_u64 ~ x: text_encoded,
    || {HeaderProp::Str(x.to_string())}));

named!(name_prop<&[u8], HeaderProp>,
  chain!(le_u64 ~ x: text_encoded,
    || {HeaderProp::Name(x.to_string())}));

named!(int_prop<&[u8], HeaderProp>,
    chain!(le_u64 ~ x: le_u32,
        || {HeaderProp::Int(x)}));

named!(bool_prop<&[u8], HeaderProp>,
    chain!(le_u64 ~ x: le_u8,
        || {HeaderProp::Bool(x == 1)}));

named!(float_prop<&[u8], HeaderProp>,
    chain!(le_u64 ~ x: le_f32,
        || {HeaderProp::Float(x)}));

named!(qword_prop<&[u8], HeaderProp>,
    chain!(le_u64 ~ x: le_u64,
        || {HeaderProp::QWord(x)}));

/// The byte property is the odd one out. It's two strings following each other. No rhyme or
/// reason.
named!(byte_prop<&[u8], HeaderProp>,
    chain!(le_u64 ~ text_encoded ~ text_encoded, || {HeaderProp::Byte}));

/// The array property has the same leading 64bits that are discarded but also contains the length
/// of the aray as the next 32bits. Each element in the array is a dictionary so we decode `size`
/// number of dictionaries.
named!(array_prop<&[u8], HeaderProp>,
    chain!(
        le_u64 ~
        size: le_u32 ~
        elems: count!(rdict, size as usize),
        || {HeaderProp::Array(elems)}));

/// The next string in the data tells us how to decode the property and what type it is.
named!(rprop_encoded<&[u8], HeaderProp>,
  switch!(text_encoded,
    "ArrayProperty" => call!(array_prop) |
    "BoolProperty" => call!(bool_prop) |
    "ByteProperty" => call!(byte_prop) |
    "FloatProperty" => call!(float_prop) |
    "IntProperty" => call!(int_prop) |
    "NameProperty" => call!(name_prop) |
    "QWordProperty" => call!(qword_prop) |
    "StrProperty" => call!(str_prop)
  )
);

/// Other the actual network data, the header property associative array is the hardest to parse.
/// The format is to:
/// - Read string
/// - If string is "None", we're done
/// - else we're dealing with a property, and the string just read is the key. Now deserialize the
///   value.
/// The return type of this function is a key value vector because since there is no format
/// specification, we can't rule out duplicate keys. Possibly consider a multi-map in the future.
fn rdict(input: &[u8]) -> IResult<&[u8], Vec<(String, HeaderProp)> > {
    let mut v: Vec<(String, HeaderProp)> = Vec::new();

    // Initialize to a dummy value to avoid unitialized errors
    let mut res: IResult<&[u8], Vec<(String, HeaderProp)>> = IResult::Done(input, Vec::new());

    // Done only if we see an error or if we see "None"
    let mut done = false;

    // Keeps track of where we currently are in the slice.
    let mut cslice = input;

    while !done {
      match text_encoded(cslice) {
        IResult::Done(i, txt) => {
          cslice = i;
          match txt {
            "None" => { done = true }
            _ => {
              match rprop_encoded(cslice) {
                IResult::Done(inp, val) => { cslice = inp; v.push((txt.to_string(), val)); },
                IResult::Incomplete(a) => { res = IResult::Incomplete(a); done = true },
                IResult::Error(a) => { res = IResult::Error(a); done = true }
              }
            }
          }
        },

        IResult::Incomplete(a) => {
          done = true;
          res = IResult::Incomplete(a);
        },

        IResult::Error(a) => {
          done = true;
          res = IResult::Error(a);
        }
      }
    }

    match res {
      IResult::Done(_, _) => IResult::Done(cslice, v),
      _ => res
    }
}

named!(pub parse<&[u8],Replay>,
    chain!(
        header_size:  le_u32 ~
        header_crc:   le_u32 ~
        major_version: le_u32 ~
        minor_version: le_u32 ~
        game_type: text_encoded ~
        properties: rdict ~
        content_size: le_u32 ~
        content_crc: le_u32 ~
        levels: text_list ~
        keyframes: keyframe_list ~
        network_size: le_u32 ~

        // This is where this example falls short is that decoding the network data is not
        // implemented. See Octane or RocketLeagueReplayParser for more info.
        take!(network_size) ~
        debug_info: debuginfo_list ~
        tick_marks: tickmark_list ~
        packages: text_list ~
        objects: text_list ~
        names: text_list ~
        class_indices: classindex_list ~
        net_cache: classnetcache_list,


        || { Replay {
          header_size: header_size,
          header_crc: header_crc,
          major_version: major_version,
          minor_version: minor_version,
          game_type: game_type.to_string(),
          properties: properties,
          content_size: content_size,
          content_crc: content_crc,
          levels: levels,
          keyframes: keyframes,
          debug_info: debug_info,
          tick_marks: tick_marks,
          packages: packages,
          objects: objects,
          names: names,
          class_indices: class_indices,
          net_cache: net_cache
        }}
    )
);

/// Below are a series of decoding functions that take in data and returns some domain object (eg:
/// TickMark, KeyFrame, etc.

named!(text_string<&[u8], String>, map!(text_encoded, str::to_string));

named!(keyframe_encoded<&[u8], KeyFrame>,
  chain!(time: le_f32 ~
         frame: le_u32 ~
         position: le_u32,
         || {KeyFrame {time: time, frame: frame, position: position}}));

named!(debuginfo_encoded<&[u8], DebugInfo>,
  chain!(frame: le_u32 ~ user: text_string ~ text: text_string,
    || { DebugInfo { frame: frame, user: user, text: text } }));

named!(tickmark_encoded<&[u8], TickMark>,
  chain!(description: text_string ~
         frame: le_u32,
         || {TickMark {description: description, frame: frame}}));

named!(classindex_encoded<&[u8], ClassIndex>,
  chain!(class: text_string ~ index: le_u32,
    || { ClassIndex { class: class, index: index } }));

named!(cacheprop_encoded<&[u8], CacheProp>,
  chain!(index: le_u32 ~ id: le_u32,
    || { CacheProp { index: index, id: id } }));

named!(classnetcache_encoded<&[u8], ClassNetCache>,
  chain!(index: le_u32 ~
         parent_id: le_u32 ~
         id: le_u32 ~
         prop_size: le_u32 ~
         properties: count!(cacheprop_encoded, prop_size as usize),
         || { ClassNetCache {
          index: index,
          parent_id: parent_id,
          id: id,
          properties: properties
         }}));

/// All the domain objects can be observed in a list that is initially prefixed by the length.
/// There may be a way to consolidate the implementations, but they're already currently concise.

named!(text_list<&[u8], Vec<String> >,
  chain!( size: le_u32 ~ elems: count!(text_string, size as usize), || {elems}));

named!(keyframe_list<&[u8], Vec<KeyFrame> >,
  chain!(size: le_u32 ~ elems: count!(keyframe_encoded, size as usize), || {elems}));

named!(debuginfo_list<&[u8], Vec<DebugInfo> >,
  chain!(size: le_u32 ~ elems: count!(debuginfo_encoded, size as usize), || {elems}));

named!(tickmark_list<&[u8], Vec<TickMark> >,
  chain!(size: le_u32 ~ elems: count!(tickmark_encoded, size as usize), || {elems}));

named!(classindex_list<&[u8], Vec<ClassIndex> >,
  chain!(size: le_u32 ~ elems: count!(classindex_encoded, size as usize), || {elems}));

named!(classnetcache_list<&[u8], Vec<ClassNetCache> >,
  chain!(size: le_u32 ~ elems: count!(classnetcache_encoded, size as usize), || {elems}));


#[cfg(test)]
mod tests {
    use nom::IResult::{Done, Error, Incomplete};
    use nom::Needed::Size;
    use super::*;
    use super::HeaderProp::*;
    use super::{length_encoded};

    #[test]
    fn missing_header_data() {
        let data = include_bytes!("../assets/rumble.replay");
        let r = length_encoded(&data[..8]);
        assert_eq!(r, Incomplete(Size(4776)));
    }

    #[test]
    fn incomplete_header_data() {
        let data = include_bytes!("../assets/rumble.replay");
        let r = length_encoded(&data[..9]);
        assert_eq!(r, Incomplete(Size(4776)));
    }

    #[test]
    fn missing_header() {
        let r = length_encoded(&[]);
        assert_eq!(r, Incomplete(Size(4)));
    }

    #[test]
    fn missing_crc_data() {
        let data = include_bytes!("../assets/rumble.replay");
        let r = length_encoded(&data[..4]);
        assert_eq!(r, Incomplete(Size(8)));
    }

    #[test]
    fn parse_a_header_with_zero_data() {
        let data = [0, 0, 0, 0, 0, 0, 0, 0];
        let r = length_encoded(&data);
        assert_eq!(r, Done(&[][..], &[][..]));
    }

    #[test]
    fn parse_text_encoding() {
        // dd skip=16 count=28 if=rumble.replay of=text.replay bs=1
        let data = include_bytes!("../assets/text.replay");
        let r = super::text_encoded(data);
        assert_eq!(r, Done(&[][..], "TAGame.Replay_Soccar_TA"));
    }

    #[test]
    fn rdict_no_elements() {
        let data = [0x05, 0x00, 0x00, 0x00, b'N', b'o', b'n', b'e', 0x00];
        let r = super::rdict(&data);
        assert_eq!(r, Done(&[][..],  Vec::new()));
    }

    #[test]
    fn rdict_one_element() {
        // dd skip=$((0x1269)) count=$((0x12a8 - 0x1269)) if=rumble.replay of=rdict_one.replay bs=1
        let data = include_bytes!("../assets/rdict_one.replay");
        let r = super::rdict(data);
        assert_eq!(r, Done(&[][..],  vec![("PlayerName".to_string(), Str("comagoosie".to_string()))]));
    }

    #[test]
    fn rdict_one_int_element() {
        // dd skip=$((0x250)) count=$((0x284 - 0x250)) if=rumble.replay of=rdict_int.replay bs=1
        let data = include_bytes!("../assets/rdict_int.replay");
        let r = super::rdict(data);
        assert_eq!(r, Done(&[][..],  vec![("PlayerTeam".to_string(), Int(0))]));
    }

    #[test]
    fn rdict_one_bool_element() {
        // dd skip=$((0xa0f)) count=$((0xa3b - 0xa0f)) if=rumble.replay of=rdict_bool.replay bs=1
        let data = include_bytes!("../assets/rdict_bool.replay");
        let r = super::rdict(data);
        assert_eq!(r, Done(&[][..],  vec![("bBot".to_string(), Bool(false))]));
    }

    fn append_none(input: &[u8]) -> Vec<u8> {
        let append = [0x05, 0x00, 0x00, 0x00, b'N', b'o', b'n', b'e', 0x00];
        let mut v = Vec::new();
        v.extend_from_slice(input);
        v.extend_from_slice(&append);
        v
    }

    #[test]
    fn rdict_one_name_element() {
        // dd skip=$((0x1237)) count=$((0x1269 - 0x1237)) if=rumble.replay of=rdict_name.replay bs=1
        let data = append_none(include_bytes!("../assets/rdict_name.replay"));
        let r = super::rdict(&data);
        assert_eq!(r, Done(&[][..],  vec![("MatchType".to_string(), Name("Online".to_string()))]));

    }

    #[test]
    fn rdict_one_float_element() {
        // dd skip=$((0x10a2)) count=$((0x10ce - 0x10a2)) if=rumble.replay of=rdict_float.replay bs=1
        let data = append_none(include_bytes!("../assets/rdict_float.replay"));
        let r = super::rdict(&data);
        assert_eq!(r, Done(&[][..],  vec![("RecordFPS".to_string(), Float(30.0))]));
    }

    #[test]
    fn rdict_one_qword_element() {
        // dd skip=$((0x576)) count=$((0x5a5 - 0x576)) if=rumble.replay of=rdict_qword.replay bs=1
        let data = append_none(include_bytes!("../assets/rdict_qword.replay"));
        let r = super::rdict(&data);
        assert_eq!(r, Done(&[][..],  vec![("OnlineID".to_string(), QWord(76561198101748375))]));
    }

    #[test]
    fn rdict_one_array_element() {
        // dd skip=$((0xab)) count=$((0x3f7 + 36)) if=rumble.replay of=rdict_array.replay bs=1
        let data = append_none(include_bytes!("../assets/rdict_array.replay"));
        let r = super::rdict(&data);
        let expected = vec![
            vec![
                ("frame".to_string(), Int(441)),
                ("PlayerName".to_string(), Str("Cakeboss".to_string())),
                ("PlayerTeam".to_string(), Int(1))
            ], vec![
                ("frame".to_string(), Int(1738)),
                ("PlayerName".to_string(), Str("Sasha Kaun".to_string())),
                ("PlayerTeam".to_string(), Int(0))
            ], vec![
                ("frame".to_string(), Int(3504)),
                ("PlayerName".to_string(), Str("SilentWarrior".to_string())),
                ("PlayerTeam".to_string(), Int(0))
            ], vec![
                ("frame".to_string(), Int(5058)),
                ("PlayerName".to_string(), Str("jeffreyj1".to_string())),
                ("PlayerTeam".to_string(), Int(1))
            ], vec![
                ("frame".to_string(), Int(5751)),
                ("PlayerName".to_string(), Str("GOOSE LORD".to_string())),
                ("PlayerTeam".to_string(), Int(0))
            ], vec![
                ("frame".to_string(), Int(6083)),
                ("PlayerName".to_string(), Str("GOOSE LORD".to_string())),
                ("PlayerTeam".to_string(), Int(0))
            ], vec![
                ("frame".to_string(), Int(7021)),
                ("PlayerName".to_string(), Str("SilentWarrior".to_string())),
                ("PlayerTeam".to_string(), Int(0))
            ]
        ];
        assert_eq!(r, Done(&[][..],  vec![("Goals".to_string(), Array(expected))]));
    }

    #[test]
    fn rdict_one_byte_element() {
        // dd skip=$((0xdf0)) count=$((0xe41 - 0xdf0)) if=rumble.replay of=rdict_byte.replay bs=1
        let data = append_none(include_bytes!("../assets/rdict_byte.replay"));
        let r = super::rdict(&data);
        assert_eq!(r, Done(&[][..],  vec![("Platform".to_string(), Byte)]));
    }

    #[test]
    fn key_frame_decode() {
        let data = include_bytes!("../assets/rumble.replay");
        let r = super::keyframe_encoded(&data[0x12da..0x12da + 12]);
        assert_eq!(r, Done(&[][..], KeyFrame { time: 16.297668, frame: 208, position: 137273 } ));
    }

    #[test]
    fn key_frame_list() {
        let data = include_bytes!("../assets/rumble.replay");

        // List is 2A long, each keyframe is 12 bytes. Then add four for list length = 508
        let r = super::keyframe_list(&data[0x12ca..0x12ca + 508]);
        match r {
          Done(i, val) => {
            // There are 42 key frames in this list
            assert_eq!(val.len(), 42);
            assert_eq!(i, &[][..]);
          }
          _ => { assert!(false); }
        }
    }

    #[test]
    fn tickmark_list() {
        let data = include_bytes!("../assets/rumble.replay");

        // 7 tick marks at 8 bytes + size of tick list
        let r = super::tickmark_list(&data[0xf6cce..0xf6d50]);
        match r {
          Done(i, val) => {
            // There are 7 tick marks in this list
            assert_eq!(val.len(), 7);
            assert_eq!(val[0], TickMark { description: "Team1Goal".to_string(), frame: 396 });
            assert_eq!(i, &[][..]);
          }
          _ => { assert!(false); }
        }
    }

}
