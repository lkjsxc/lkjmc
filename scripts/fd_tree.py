#!/usr/bin/env python3
"""Deterministic descriptor-relative, no-follow tree traversal."""
from __future__ import annotations
import os,stat
from dataclasses import dataclass
from pathlib import Path,PurePosixPath
from typing import Callable

@dataclass(frozen=True)
class Limits:
 max_entries:int=10000
 max_files:int=10000
 max_bytes:int=3*1024*1024*1024
 max_file_bytes:int=2*1024*1024*1024
 max_depth:int=16

@dataclass(frozen=True)
class Entry:
 path:str
 size:int
 mode:int

def fail(message:str)->None: raise RuntimeError(message)
def identity(value:os.stat_result)->tuple[int,...]:
 return (value.st_dev,value.st_ino,value.st_mode,value.st_nlink,value.st_size,
  value.st_mtime_ns,value.st_ctime_ns)
def same(before:os.stat_result,after:os.stat_result,label:str)->None:
 if identity(before)!=identity(after): fail(f'traversal race changed {label}')
def private_mode(value:os.stat_result,directory:bool,label:str)->None:
 mode=stat.S_IMODE(value.st_mode); allowed=(0o700,) if directory else (0o600,0o700)
 if mode not in allowed: fail(f'permission violation at {label}: {mode:04o}')
def open_root(path:Path)->tuple[int,os.stat_result]:
 try: before=os.lstat(path)
 except OSError as error: fail(f'unstatable root {path}: {error}')
 if stat.S_ISLNK(before.st_mode): fail(f'symlink root: {path}')
 if not stat.S_ISDIR(before.st_mode): fail(f'root is not a directory: {path}')
 private_mode(before,True,'.')
 flags=os.O_RDONLY|os.O_CLOEXEC|os.O_DIRECTORY|os.O_NOFOLLOW
 try: descriptor=os.open(path,flags)
 except OSError as error: fail(f'unreadable root {path}: {error}')
 after=os.fstat(descriptor)
 try: same(before,after,'.')
 except Exception: os.close(descriptor); raise
 return descriptor,after

def visit_file(path:Path,visit:Callable[[int,Entry],None],limits:Limits=Limits())->Entry:
 path=Path(path)
 try: before=os.lstat(path)
 except OSError as error: fail(f'unstatable file {path}: {error}')
 if not stat.S_ISREG(before.st_mode) or stat.S_ISLNK(before.st_mode): fail(f'nonregular file: {path}')
 private_mode(before,False,path.name)
 if before.st_size>limits.max_file_bytes or before.st_size>limits.max_bytes: fail(f'byte overflow at {path.name}')
 try: descriptor=os.open(path,os.O_RDONLY|os.O_CLOEXEC|os.O_NOFOLLOW)
 except OSError as error: fail(f'unreadable file {path}: {error}')
 try:
  current=os.fstat(descriptor); same(before,current,path.name)
  entry=Entry(path.name,before.st_size,stat.S_IMODE(before.st_mode)); visit(descriptor,entry)
  same(current,os.fstat(descriptor),path.name); same(before,os.lstat(path),path.name)
  return entry
 finally: os.close(descriptor)

def walk(root:Path,visit:Callable[[int,Entry],None],limits:Limits=Limits(),
         visit_directory:Callable[[int,Entry],None]|None=None)->list[Entry]:
 """Visit each regular file while its verified descriptor remains open."""
 root=Path(root); root_fd,root_stat=open_root(root); entries=[]; totals=[0,0,0]
 def descend(directory:int,label:str,depth:int,opened:os.stat_result)->None:
  if depth>limits.max_depth: fail(f'depth overflow at {label}')
  try: names=sorted(os.listdir(directory),key=os.fsencode)
  except OSError as error: fail(f'unreadable directory {label}: {error}')
  if len(names)!=len(set(names)): fail(f'duplicate directory entry at {label}')
  totals[0]+=len(names)
  if totals[0]>limits.max_entries: fail('entry count overflow')
  for name in names:
   if name in ('.','..') or '/' in name or '\x00' in name: fail(f'invalid entry at {label}')
   relative=name if label=='.' else f'{label}/{name}'
   if str(PurePosixPath(relative))!=relative: fail(f'noncanonical entry: {relative}')
   try: before=os.stat(name,dir_fd=directory,follow_symlinks=False)
   except OSError as error: fail(f'unstatable entry {relative}: {error}')
   if before.st_dev!=root_stat.st_dev: fail(f'root crossing at {relative}')
   if stat.S_ISLNK(before.st_mode): fail(f'symlink entry: {relative}')
   if stat.S_ISDIR(before.st_mode):
    private_mode(before,True,relative)
    try: child=os.open(name,os.O_RDONLY|os.O_CLOEXEC|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=directory)
    except OSError as error: fail(f'unreadable directory {relative}: {error}')
    try:
     current=os.fstat(child); same(before,current,relative)
     if visit_directory is not None:
      visit_directory(child,Entry(relative,0,stat.S_IMODE(before.st_mode)))
     descend(child,relative,depth+1,current)
     same(current,os.fstat(child),relative)
     same(before,os.stat(name,dir_fd=directory,follow_symlinks=False),relative)
    finally: os.close(child)
   elif stat.S_ISREG(before.st_mode):
    private_mode(before,False,relative); totals[1]+=1; totals[2]+=before.st_size
    if totals[1]>limits.max_files: fail('file count overflow')
    if before.st_size>limits.max_file_bytes or totals[2]>limits.max_bytes: fail(f'byte overflow at {relative}')
    try: child=os.open(name,os.O_RDONLY|os.O_CLOEXEC|os.O_NOFOLLOW,dir_fd=directory)
    except OSError as error: fail(f'unreadable file {relative}: {error}')
    try:
     current=os.fstat(child); same(before,current,relative)
     entry=Entry(relative,before.st_size,stat.S_IMODE(before.st_mode)); visit(child,entry)
     same(current,os.fstat(child),relative)
     same(before,os.stat(name,dir_fd=directory,follow_symlinks=False),relative)
     entries.append(entry)
    finally: os.close(child)
   else: fail(f'special file: {relative}')
  try: final=sorted(os.listdir(directory),key=os.fsencode)
  except OSError as error: fail(f'unreadable directory {label}: {error}')
  if names!=final: fail(f'traversal race changed directory {label}')
 try:
  descend(root_fd,'.',0,root_stat); same(root_stat,os.fstat(root_fd),'.')
  same(root_stat,os.lstat(root),'.')
 finally: os.close(root_fd)
 return entries

def handoff(root:Path,uid:int,gid:int,limits:Limits=Limits())->list[Entry]:
 """Validate a tree and atomically normalize ownership/modes by descriptor."""
 root=Path(root)
 try: before=os.lstat(root)
 except OSError as error: fail(f'unstatable root {root}: {error}')
 if not stat.S_ISDIR(before.st_mode) or stat.S_ISLNK(before.st_mode): fail(f'invalid handoff root: {root}')
 try: root_fd=os.open(root,os.O_RDONLY|os.O_CLOEXEC|os.O_DIRECTORY|os.O_NOFOLLOW)
 except OSError as error: fail(f'unreadable root {root}: {error}')
 root_stat=os.fstat(root_fd); same(before,root_stat,'.'); entries=[]; totals=[0,0]
 def descend(directory:int,label:str,depth:int)->None:
  if depth>limits.max_depth: fail(f'depth overflow at {label}')
  try: names=sorted(os.listdir(directory),key=os.fsencode)
  except OSError as error: fail(f'unreadable directory {label}: {error}')
  totals[0]+=len(names)
  if totals[0]>limits.max_entries: fail('entry count overflow')
  for name in names:
   relative=name if label=='.' else f'{label}/{name}'
   try: initial=os.stat(name,dir_fd=directory,follow_symlinks=False)
   except OSError as error: fail(f'unstatable entry {relative}: {error}')
   if initial.st_dev!=root_stat.st_dev: fail(f'root crossing at {relative}')
   directory_entry=stat.S_ISDIR(initial.st_mode)
   if not directory_entry and not stat.S_ISREG(initial.st_mode): fail(f'symlink or special file: {relative}')
   flags=os.O_RDONLY|os.O_CLOEXEC|os.O_NOFOLLOW|(os.O_DIRECTORY if directory_entry else 0)
   try: child=os.open(name,flags,dir_fd=directory)
   except OSError as error: fail(f'unreadable entry {relative}: {error}')
   try:
    same(initial,os.fstat(child),relative)
    if directory_entry: descend(child,relative,depth+1)
    else:
     totals[1]+=initial.st_size
     if initial.st_size>limits.max_file_bytes or totals[1]>limits.max_bytes: fail(f'byte overflow at {relative}')
     entries.append(Entry(relative,initial.st_size,0o600))
    os.fchown(child,uid,gid); os.fchmod(child,0o700 if directory_entry else 0o600)
    final=os.fstat(child); same(final,os.stat(name,dir_fd=directory,follow_symlinks=False),relative)
   finally: os.close(child)
  if names!=sorted(os.listdir(directory),key=os.fsencode): fail(f'traversal race changed directory {label}')
 try:
  descend(root_fd,'.',0); os.fchown(root_fd,uid,gid); os.fchmod(root_fd,0o700)
  final=os.fstat(root_fd); same(final,os.lstat(root),'.')
 finally: os.close(root_fd)
 return entries
