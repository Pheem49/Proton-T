import curses
import os
from proton_t import core

def draw_menu(stdscr, selected_idx, matches):
    stdscr.clear()
    h, w = stdscr.getmaxyx()
    
    # Split column
    list_w = int(w * 0.4)
    preview_w = w - list_w - 2
    
    # Title
    title = " Proton-T Selection (Arrows: Move, Enter: Select, q: Quit) "
    stdscr.attron(curses.A_REVERSE)
    stdscr.addstr(0, max(0, (w - len(title)) // 2), title[:w])
    stdscr.attroff(curses.A_REVERSE)

    # Draw separator
    for y in range(1, h - 1):
        if list_w < w:
            stdscr.addstr(y, list_w, "│")

    # Draw matches list
    for i, path in enumerate(matches[:h-3]):
        y = i + 1
        if i == selected_idx:
            stdscr.attron(curses.A_REVERSE)
            # Truncate and pad
            display_text = f"> {path}"
            if len(display_text) > list_w - 1:
                display_text = display_text[:list_w-4] + "..."
            stdscr.addstr(y, 1, display_text.ljust(list_w - 1))
            stdscr.attroff(curses.A_REVERSE)
        else:
            display_text = f"  {path}"
            if len(display_text) > list_w - 1:
                display_text = display_text[:list_w-4] + "..."
            stdscr.addstr(y, 1, display_text[:list_w-1])

    # Draw Preview
    try:
        if matches and selected_idx < len(matches):
            selected_path = matches[selected_idx]
            if 1 < h - 1:
                stdscr.addstr(1, list_w + 2, f"Preview: {os.path.basename(selected_path)}"[:preview_w], curses.A_BOLD)
            if 2 < h - 1:
                stdscr.addstr(2, list_w + 2, "─" * (min(preview_w, len(os.path.basename(selected_path)) + 9)))
            
            preview_items = core.get_directory_preview(selected_path, limit=h-5)
            for i, item in enumerate(preview_items):
                if 3 + i < h - 1:
                    stdscr.addstr(3 + i, list_w + 2, item[:preview_w])
    except curses.error:
        pass # Ignore errors from drawing out of bounds

    stdscr.refresh()

def run_tui(matches):
    if not matches:
        return None
        
    def main(stdscr):
        # Setup curses
        curses.curs_set(0)
        curses.use_default_colors()
        if curses.has_colors():
            # Define some basic colors if available
            curses.init_pair(1, curses.COLOR_CYAN, -1) # Selection
        
        selected_idx = 0
        
        while True:
            draw_menu(stdscr, selected_idx, matches)
            
            key = stdscr.getch()
            
            if key == curses.KEY_UP:
                selected_idx = max(0, selected_idx - 1)
            elif key == curses.KEY_DOWN:
                selected_idx = min(len(matches) - 1, selected_idx + 1)
            elif key in (curses.KEY_ENTER, 10, 13):
                return matches[selected_idx]
            elif key in (ord('q'), 27): # 'q' or Esc
                return None
            elif key == curses.KEY_RESIZE:
                stdscr.clear()

    return curses.wrapper(main)
