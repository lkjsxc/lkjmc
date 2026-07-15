#!/usr/bin/env python3
"""Hand a retained tree to one owner after descriptor-safe closure validation."""
import argparse,os,sys
from pathlib import Path
from fd_tree import Limits,handoff

def main():
 parser=argparse.ArgumentParser(); parser.add_argument('root'); parser.add_argument('--owner')
 args=parser.parse_args(); owner=args.owner or f'{os.getuid()}:{os.getgid()}'
 try: uid,gid=map(int,owner.split(':',1))
 except ValueError: raise RuntimeError('owner must be numeric uid:gid')
 entries=handoff(Path(args.root),uid,gid,Limits(max_entries=100000,max_files=100000,
  max_bytes=3*1024**3,max_file_bytes=2*1024**3,max_depth=32))
 print(f'ok private-artifact-handoff files={len(entries)}')
if __name__=='__main__':
 try: main()
 except Exception as error: print(f'artifact handoff failed: {error}',file=sys.stderr); sys.exit(1)
