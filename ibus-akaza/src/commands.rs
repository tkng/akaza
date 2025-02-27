use std::collections::HashMap;

use ibus_sys::engine::IBusEngine;

use crate::input_mode::{
    INPUT_MODE_ALNUM, INPUT_MODE_FULLWIDTH_ALNUM, INPUT_MODE_HALFWIDTH_KATAKANA,
    INPUT_MODE_HIRAGANA, INPUT_MODE_KATAKANA,
};
use crate::AkazaContext;

/**
 * shortcut key を設定可能な機能。
 */
pub type IbusAkazaCommand = fn(&mut AkazaContext, *mut IBusEngine);

pub(crate) fn ibus_akaza_commands_map() -> HashMap<&'static str, IbusAkazaCommand> {
    let mut function_map: HashMap<&'static str, IbusAkazaCommand> = HashMap::new();

    // shorthand
    let mut register = |name: &'static str, cmd: IbusAkazaCommand| function_map.insert(name, cmd);

    register("commit_candidate", |context, engine| {
        context.commit_candidate(engine);
    });
    // 無変換状態では、ひらがなに変換してコミットします
    register("commit_preedit", |context, engine| {
        let (_, surface) = context.make_preedit_word();
        context.commit_string(engine, surface.as_str());
    });
    register("escape", |context, engine| context.escape(engine));
    register("page_up", |context, engine| {
        context.page_up(engine);
    });
    register("page_down", |context, engine| {
        context.page_down(engine);
    });

    register("set_input_mode_hiragana", |context, engine| {
        context.set_input_mode(engine, &INPUT_MODE_HIRAGANA)
    });
    register("set_input_mode_alnum", |context, engine| {
        context.set_input_mode(engine, &INPUT_MODE_ALNUM)
    });
    register("set_input_mode_fullwidth_alnum", |context, engine| {
        context.set_input_mode(engine, &INPUT_MODE_FULLWIDTH_ALNUM)
    });
    register("set_input_mode_katakana", |context, engine| {
        context.set_input_mode(engine, &INPUT_MODE_KATAKANA)
    });
    register("set_input_mode_halfwidth_katakana", |context, engine| {
        context.set_input_mode(engine, &INPUT_MODE_HALFWIDTH_KATAKANA)
    });

    register("update_candidates", |context, engine| {
        context.update_candidates(engine)
    });
    register("erase_character_before_cursor", |context, engine| {
        context.erase_character_before_cursor(engine)
    });
    register("cursor_up", |context, engine| {
        context.cursor_up(engine);
    });
    register("cursor_down", |context, engine| {
        context.cursor_down(engine);
    });
    register("cursor_right", |context, engine| {
        context.cursor_right(engine);
    });
    register("cursor_left", |context, engine| {
        context.cursor_left(engine);
    });
    register("extend_clause_right", |context, engine| {
        context.extend_clause_right(engine).unwrap();
    });
    register("extend_clause_left", |context, engine| {
        context.extend_clause_left(engine).unwrap();
    });
    register("convert_to_full_hiragana", |context, engine| {
        context.convert_to_full_hiragana(engine).unwrap();
    });
    register("convert_to_full_katakana", |context, engine| {
        context.convert_to_full_katakana(engine).unwrap();
    });
    register("convert_to_half_katakana", |context, engine| {
        context.convert_to_half_katakana(engine).unwrap();
    });
    register("convert_to_full_romaji", |context, engine| {
        context.convert_to_full_romaji(engine).unwrap();
    });
    register("convert_to_half_romaji", |context, engine| {
        context.convert_to_half_romaji(engine).unwrap();
    });

    {
        // TODO コピペがすごい。マクロかうまいなにかでまとめて登録できるようにしたい。
        register("press_number_1", |context, engine| {
            context.process_num_key(1, engine);
        });
        register("press_number_2", |context, engine| {
            context.process_num_key(2, engine);
        });
        register("press_number_3", |context, engine| {
            context.process_num_key(3, engine);
        });
        register("press_number_4", |context, engine| {
            context.process_num_key(4, engine);
        });
        register("press_number_5", |context, engine| {
            context.process_num_key(5, engine);
        });
        register("press_number_6", |context, engine| {
            context.process_num_key(6, engine);
        });
        register("press_number_7", |context, engine| {
            context.process_num_key(7, engine);
        });
        register("press_number_8", |context, engine| {
            context.process_num_key(8, engine);
        });
        register("press_number_9", |context, engine| {
            context.process_num_key(9, engine);
        });
        register("press_number_0", |context, engine| {
            context.process_num_key(0, engine);
        });
    }

    function_map
}
