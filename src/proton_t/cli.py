import sys, argparse
from proton_t import core

def main():
    parser = argparse.ArgumentParser(description="Proton-T CLI")
    sub = parser.add_subparsers(dest="command")
    sub.add_parser("add").add_argument("path")
    sub.add_parser("query").add_argument("keywords", nargs="+")
    sub.add_parser("list")
    sub.add_parser("interactive").add_argument("keywords", nargs="*", default=[])
    args = parser.parse_args()
    
    if args.command == "add": core.add_path(args.path)
    elif args.command == "query":
        res = core.query_paths(args.keywords)
        if res: print(res)
        else: sys.exit(1)
    elif args.command == "list":
        for p, r, s in core.list_paths(): print(f"{r:.2f}\t{s}\t{p}")
    elif args.command == "interactive":
        matches = core.get_all_matches(args.keywords) if args.keywords else [p for p, r, s in core.list_paths()[:10]]
        if not matches: sys.exit(1)
        
        if len(matches) == 1:
            print(matches[0])
            return

        sys.stderr.write("Select directory:\n")
        for i, path in enumerate(matches[:10], 1):
            sys.stderr.write(f"  {i}) {path}\n")
        sys.stderr.write(f"\nSelection [1-{min(len(matches), 10)}, q to quit]: ")
        sys.stderr.flush()
        
        try:
            choice = sys.stdin.readline().strip().lower()
            if not choice or choice == 'q': sys.exit(0)
            idx = int(choice) - 1
            if 0 <= idx < len(matches):
                print(matches[idx])
                core.add_path(matches[idx])
            else:
                sys.exit(1)
        except (EOFError, KeyboardInterrupt):
            sys.stderr.write("\n")
            sys.exit(0)
        except:
            sys.exit(1)

if __name__ == "__main__": main()
