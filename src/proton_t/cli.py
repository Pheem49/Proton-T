import sys, argparse
from proton_t import core, __version__

# ANSI escape codes for basic cross-platform colors
CYAN = '\033[96m'
GREEN = '\033[92m'
YELLOW = '\033[93m'
RESET = '\033[0m'

def main():
    parser = argparse.ArgumentParser(description="Proton-T CLI")
    parser.add_argument("-v", "--version", action="version", version=f"Proton-T {__version__}")
    sub = parser.add_subparsers(dest="command")
    
    sub.add_parser("add").add_argument("path")
    sub.add_parser("query").add_argument("keywords", nargs="+")
    sub.add_parser("list")
    
    sub.add_parser("remove", help="Remove a directory from the tracking database").add_argument("path")
    sub.add_parser("clean", help="Remove invalid directories from the tracking database")
    
    sub.add_parser("interactive").add_argument("keywords", nargs="*", default=[])
    args = parser.parse_args()
    
    if args.command == "add": core.add_path(args.path)
    elif args.command == "query":
        res = core.query_paths(args.keywords)
        if res: print(res)
        else: sys.exit(1)
    elif args.command == "remove":
        if core.remove_path(args.path):
            print(f"{GREEN}Removed '{args.path}' from database.{RESET}")
        else:
            print(f"{YELLOW}Path '{args.path}' not found in database.{RESET}")
            sys.exit(1)
            
    elif args.command == "clean":
        removed_count = core.clean_db()
        print(f"{GREEN}Cleaned {removed_count} invalid paths from the database.{RESET}")
        
    elif args.command == "list":
        all_paths = core.list_paths()
        if not all_paths: return
        
        projects = [p for p in all_paths if p[2].get('is_project')]
        projects.sort(key=lambda x: x[1], reverse=True)
        recent = sorted(all_paths, key=lambda x: x[2]['last_access'], reverse=True)
        frequent = sorted(all_paths, key=lambda x: x[2]['score'], reverse=True)
        
        shown = set()
        def print_section(title, items):
            count = 0
            has_title = False
            for p, r, e in items:
                if count >= 3: break
                if p not in shown:
                    if not has_title:
                        print(f"\n{CYAN}{title}{RESET}")
                        has_title = True
                    print(f"  - {GREEN}{p}{RESET}")
                    shown.add(p)
                    count += 1
        
        print_section("[Suggested Projects]", projects)
        print_section("[Recent Paths]", recent)
        print_section("[Frequent Paths]", frequent)
        print("") # Final newline
        
    elif args.command == "interactive":
        all_paths = core.list_paths()
        valid_keywords = [k for k in args.keywords if k.strip()]
        
        if valid_keywords:
            matches = core.get_all_matches(valid_keywords)
        else:
            matches = []
            
        if not matches and not valid_keywords and all_paths:
            # Grouped interactive
            projects = [p for p in all_paths if p[2].get('is_project')]
            projects.sort(key=lambda x: x[1], reverse=True)
            recent = sorted(all_paths, key=lambda x: x[2]['last_access'], reverse=True)
            frequent = sorted(all_paths, key=lambda x: x[2]['score'], reverse=True)
            
            sys.stderr.write(f"{YELLOW}Select directory:{RESET}\n")
            def add_menu_section(title, items):
                count = 0
                has_title = False
                for p, r, e in items:
                    if count >= 3: break
                    if p not in matches:
                        if not has_title:
                            sys.stderr.write(f"\n{CYAN}{title}{RESET}\n")
                            has_title = True
                        matches.append(p)
                        sys.stderr.write(f"  {len(matches)}) {GREEN}{p}{RESET}\n")
                        count += 1
            
            add_menu_section("[Suggested Projects]", projects)
            add_menu_section("[Recent Paths]", recent)
            add_menu_section("[Frequent Paths]", frequent)
        elif matches:
            sys.stderr.write(f"{YELLOW}Select directory:{RESET}\n")
            for i, path in enumerate(matches[:10], 1):
                sys.stderr.write(f"  {i}) {GREEN}{path}{RESET}\n")
        
        if not matches: sys.exit(1)
        
        if len(matches) == 1 and args.keywords:
            print(matches[0])
            return

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
