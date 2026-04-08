import sys, argparse
from proton_t import core, tui

def main():
    parser = argparse.ArgumentParser(description="Proton-T CLI")
    sub = parser.add_subparsers(dest="command")
    sub.add_parser("add").add_argument("path")
    sub.add_parser("query").add_argument("keywords", nargs="+")
    sub.add_parser("list")
    sub.add_parser("interactive").add_argument("keywords", nargs="*", default=[])
    sub.add_parser("ls")
    args = parser.parse_args()
    
    if args.command == "add": core.add_path(args.path)
    elif args.command == "query":
        res = core.query_paths(args.keywords)
        if res: print(res)
        else: sys.exit(1)
    elif args.command == "list":
        for p, r, s in core.list_paths(): print(f"{r:.2f}\t{s}\t{p}")
    elif args.command == "ls":
        entries = core.list_current_dir_with_scores()
        for e in entries:
            name = e['name']
            if e['is_dir']:
                name += "/"
                if e['is_tracked']:
                    # Highlight tracked directories
                    color = "\033[1;32m" # Bold Green
                    score_info = f" ({e['score']:.1f})"
                else:
                    color = "\033[1;34m" # Bold Blue
                    score_info = ""
                print(f"{color}{name:<20}\033[0m{score_info}")
            else:
                print(f"{name}")
    elif args.command == "interactive":
        matches = core.get_all_matches(args.keywords) if args.keywords else [p for p, r, s in core.list_paths()[:10]]
        if not matches: sys.exit(1)
        
        if len(matches) == 1:
            print(matches[0])
            return

        selected = tui.run_tui(matches)
        if selected:
            print(selected)
            core.add_path(selected)
        else:
            sys.exit(0)

if __name__ == "__main__": main()
