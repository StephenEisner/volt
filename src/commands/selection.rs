use crate::models::application::{Application, ClipboardContent};
use scribe::buffer::{LineRange, Range};
use super::application;
use crate::errors::*;
use crate::commands::{self, Result};
use crate::util;
use crate::models::application::modes::{SelectLineMode, SearchMode, SelectMode};

pub fn delete(app: &mut Application) -> Result {
    let mut curr_mode = app.mode_stack 
        .pop_front() 
        .unwrap();
    if let Some(buffer) = app.workspace.current_buffer() {
        match curr_mode.mode_id() {
            Some("select") => {
                let cursor_position = *buffer.cursor.clone();
                let delete_range = Range::new(cursor_position, curr_mode
                                              .as_any_mut()
                                              .downcast_mut::<SelectMode>()
                                              .unwrap()
                                              .anchor);
                buffer.delete_range(delete_range.clone());
                buffer.cursor.move_to(delete_range.start());
            }
            Some("select_line") => {
                let delete_range = curr_mode
                    .as_any()
                    .downcast_ref::<SelectLineMode>()
                    .unwrap()
                    .to_range(&*buffer.cursor);
                buffer.delete_range(delete_range.clone());
                buffer.cursor.move_to(delete_range.start());
            }
            Some("search") => {
                let selection = curr_mode
                    .as_any()
                    .downcast_ref::<SearchMode>()
                    .unwrap()
                    .results
                    .as_ref()
                    .and_then(|r| r.selection())
                    .ok_or("Can't delete in search mode without a selected result")?;
                buffer.delete_range(selection.clone());
            }
            _ => bail!("Can't delete selections outside of select mode"),
        };
    } else {
        bail!(BUFFER_MISSING);
    }
    app.mode_stack.push_front(curr_mode);

    Ok(())
}

pub fn copy_and_delete(app: &mut Application) -> Result {
    let _ = copy_to_clipboard(app);
    delete(app)
}

pub fn change(app: &mut Application) -> Result {
    let _ = copy_to_clipboard(app);
    delete(app)?;
    application::switch_to_insert_mode(app)?;
    commands::view::scroll_to_cursor(app)
}

pub fn copy(app: &mut Application) -> Result {
    copy_to_clipboard(app)?;
    application::switch_to_normal_mode(app)
}

pub fn select_all(app: &mut Application) -> Result {
    app.workspace
        .current_buffer()
        .ok_or(BUFFER_MISSING)?
        .cursor
        .move_to_first_line();
    application::switch_to_select_line_mode(app)?;
    app.workspace
        .current_buffer()
        .ok_or(BUFFER_MISSING)?
        .cursor
        .move_to_last_line();

    Ok(())
}

fn copy_to_clipboard(app: &mut Application) -> Result {
    let curr_mode = app.mode_stack 
        .pop_front() 
        .unwrap();

    let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;

    match curr_mode.mode_id() {
        Some("select") => {
            let cursor_position = *buffer.cursor.clone();
            let selected_range = Range::new(cursor_position, curr_mode
                                            .as_any()
                                            .downcast_ref::<SelectMode>()
                                            .unwrap()
                                            .anchor);

            let data = buffer.read(&selected_range.clone())
                .ok_or("Couldn't read selected data from buffer")?;
            app.clipboard.set_content(ClipboardContent::Inline(data))?;
        }
        Some("select_line") => {
            let selected_range = util::inclusive_range(
                &LineRange::new(
                    curr_mode
                    .as_any()
                    .downcast_ref::<SelectLineMode>()
                    .unwrap()
                    .anchor,
                    buffer.cursor
                    .line
                ),
                buffer
            );

            let data = buffer.read(&selected_range.clone())
                .ok_or("Couldn't read selected data from buffer")?;
            app.clipboard.set_content(ClipboardContent::Block(data))?;
        }
        _ => bail!("Can't copy data to clipboard outside of select modes"),
    };
    app.mode_stack.push_front(curr_mode);

    Ok(())
}

//{#[cfg(test)]
//mod tests {
//    use crate::commands;
//    use crate::models::application::{Application, Mode};
//    use scribe::Buffer;
//    use scribe::buffer::Position;
//
//    #[test]
//    fn select_all_selects_the_entire_buffer() {
//        let mut app = Application::new(&Vec::new()).unwrap();
//        let mut buffer = Buffer::new();
//
//        // Insert data with indentation and move to the end of the line.
//        buffer.insert("amp\neditor\nbuffer");
//        let position = Position {
//            line: 1,
//            offset: 3,
//        };
//        buffer.cursor.move_to(position);
//
//        // Now that we've set up the buffer, add it
//        // to the application and call the command.
//        app.workspace.add_buffer(buffer);
//        super::select_all(&mut app).unwrap();
//
//        // Ensure that the application is in select line mode,
//        // and that its anchor position is on the first line
//        // of the buffer.
//        match app.mode {
//            Mode::SelectLine(ref mode) => {
//                assert_eq!(mode.anchor, 0);
//            },
//            _ => panic!("Application isn't in select line mode.")
//        }
//
//        // Ensure that the cursor is moved to the last line of the buffer.
//        assert_eq!(app.workspace.current_buffer().unwrap().cursor.line, 2);
//    }
//
//    #[test]
//    fn delete_removes_the_selection_in_select_mode() {
//        let mut app = Application::new(&Vec::new()).unwrap();
//        let mut buffer = Buffer::new();
//
//        // Insert data with indentation and move to the end of the line.
//        buffer.insert("amp\neditor\nbuffer");
//        let position = Position {
//            line: 1,
//            offset: 0,
//        };
//        buffer.cursor.move_to(position);
//
//        // Now that we've set up the buffer, add it
//        // to the application and call the command.
//        app.workspace.add_buffer(buffer);
//        commands::application::switch_to_select_mode(&mut app).unwrap();
//        commands::cursor::move_right(&mut app).unwrap();
//        commands::selection::delete(&mut app).unwrap();
//
//        // Ensure that the cursor is moved to the last line of the buffer.
//        assert_eq!(
//            app.workspace.current_buffer().unwrap().data(),
//            String::from("amp\nditor\nbuffer")
//        )
//    }
//
//    #[test]
//    fn delete_removes_the_selected_line_in_select_line_mode() {
//        let mut app = Application::new(&Vec::new()).unwrap();
//        let mut buffer = Buffer::new();
//
//        // Insert data with indentation and move to the end of the line.
//        buffer.insert("amp\neditor\nbuffer");
//        let position = Position {
//            line: 1,
//            offset: 0,
//        };
//        buffer.cursor.move_to(position);
//
//        // Now that we've set up the buffer, add it
//        // to the application and call the command.
//        app.workspace.add_buffer(buffer);
//        commands::application::switch_to_select_line_mode(&mut app).unwrap();
//        commands::selection::delete(&mut app).unwrap();
//
//        // Ensure that the cursor is moved to the last line of the buffer.
//        assert_eq!(
//            app.workspace.current_buffer().unwrap().data(),
//            String::from("amp\nbuffer")
//        )
//    }
//
//    #[test]
//    fn delete_removes_the_current_result_in_search_mode() {
//        let mut app = Application::new(&Vec::new()).unwrap();
//        let mut buffer = Buffer::new();
//
//        // Insert data with indentation and move to the end of the line.
//        buffer.insert("amp\neditor\nbuffer");
//        let position = Position {
//            line: 1,
//            offset: 0,
//        };
//        buffer.cursor.move_to(position);
//
//        // Now that we've set up the buffer, add it
//        // to the application and call the command.
//        app.workspace.add_buffer(buffer);
//        app.search_query = Some(String::from("ed"));
//        commands::application::switch_to_search_mode(&mut app).unwrap();
//        commands::search::accept_query(&mut app).unwrap();
//        commands::selection::delete(&mut app).unwrap();
//
//        // Ensure that the cursor is moved to the last line of the buffer.
//        assert_eq!(
//            app.workspace.current_buffer().unwrap().data(),
//            String::from("amp\nitor\nbuffer")
//        )
//    }
//}
//}
