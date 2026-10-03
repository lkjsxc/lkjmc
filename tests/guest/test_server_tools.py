"""Adversarial paths and crash-replay tests without systemd, Java or a host."""
import gzip
import hashlib
import json
import os
import threading
import time
import unittest
import uuid
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch
import test_guest
from test_guest import PowerLoss, guest

class ServerTools(unittest.TestCase):
    setUp = test_guest.GuestFiles.setUp
    archive = test_guest.GuestFiles.archive
    request = test_guest.GuestFiles.request
    def change(self, path='notes.txt', text='hello', expected_sha256=None):
        return dict(job_id=str(uuid.uuid4()),path=path,text=text,expected_sha256=expected_sha256)

    def test_text_create_edit_delete_replays_after_effect_before_receipt(self):
        for action in ('file_write','file_delete','directory_create'):
            with self.subTest(action=action):
                path=action+'.txt'
                req=self.change(path)
                if action!='directory_create':
                    (guest.ROOT/path).write_text('old')
                    req['expected_sha256']=hashlib.sha256(b'old').hexdigest()
                original=guest.atomic
                def crash(path,value):
                    if value['phase']=='committed':raise PowerLoss()
                    original(path,value)
                with patch.object(guest,'atomic',crash),self.assertRaises(PowerLoss):
                    guest.mutation(req,action)
                result=guest.mutation(req,action)
                self.assertEqual(result['effect'],'committed')
                self.assertEqual(guest.mutation(req,action),result)
                if action=='file_delete':self.assertFalse((guest.ROOT/path).exists())
                elif action=='directory_create':self.assertTrue((guest.ROOT/path).is_dir())
                else:self.assertEqual(guest.file_read({'path':path})['text'],'hello')
                with self.assertRaises(ValueError):guest.mutation({**req,'text':'changed'},action)

    def test_stale_guards_and_committed_receipt_never_mask_changed_effect(self):
        req=self.change()
        result=guest.mutation(req,'file_write')
        self.assertEqual(result['sha256'],hashlib.sha256(b'hello').hexdigest())
        with self.assertRaises(ValueError):guest.mutation(self.change(),'file_write')
        with self.assertRaises(ValueError):guest.mutation(self.change(expected_sha256='0'*64),'file_delete')
        (guest.ROOT/'notes.txt').write_text('newer')
        with self.assertRaises(ValueError):guest.mutation(req,'file_write')
        self.assertEqual((guest.ROOT/'notes.txt').read_text(),'newer')
        (guest.ROOT/'world').mkdir()
        with self.assertRaises(ValueError):guest.mutation(self.change('world',expected_sha256='0'*64),'file_delete')
        (guest.ROOT/'level.dat').write_text('world')
        with self.assertRaises(ValueError):guest.mutation(self.change('level.dat',expected_sha256=hashlib.sha256(b'world').hexdigest()),'file_delete')

    def test_protected_paths_links_special_files_and_non_utf8_never_exposed(self):
        outside=self.root/'outside.txt';outside.write_text('secret')
        (guest.ROOT/'linked.txt').symlink_to(outside)
        os.link(outside,guest.ROOT/'hard.txt')
        os.mkfifo(guest.ROOT/'pipe.txt')
        for name in ('server.properties','token.txt','ops.json','eula.txt'):(guest.ROOT/name).write_text('secret')
        for name in ('config','plugins'):
            (guest.ROOT/name).mkdir();(guest.ROOT/name/'innocent.txt').write_text('secret')
        (guest.ROOT/'ok.txt').write_text('visible')
        self.assertEqual([v['name'] for v in guest.files({'path':''})['entries']],['ok.txt'])
        for name in ('linked.txt','hard.txt','pipe.txt','server.properties','config/innocent.txt','ops.json'):
            with self.subTest(name=name):
                with self.assertRaises((OSError,ValueError)):guest.file_read({'path':name})
                with self.assertRaises((OSError,ValueError)):guest.mutation(self.change(name),'file_write')
                with self.assertRaises((OSError,ValueError)):guest.install(self.request(target=name))
        (guest.ROOT/'binary.txt').write_bytes(b'\xff\x00')
        with self.assertRaises(ValueError):guest.file_read({'path':'binary.txt'})
        self.assertEqual(outside.read_text(),'secret')

    def test_directory_symlink_swap_races_never_read_outside_root(self):
        (guest.ROOT/'folder').mkdir();(guest.ROOT/'folder'/'data.txt').write_text('safe')
        external=self.root/'external';external.mkdir();(external/'data.txt').write_text('PRIVATE')
        stop=threading.Event()
        def swaps():
            while not stop.is_set():
                os.rename(guest.ROOT/'folder',guest.ROOT/'parked')
                (guest.ROOT/'folder').symlink_to(external)
                (guest.ROOT/'folder').unlink()
                os.rename(guest.ROOT/'parked',guest.ROOT/'folder')
        thread=threading.Thread(target=swaps);thread.start()
        try:
            for _ in range(150):
                try:self.assertEqual(guest.file_read({'path':'folder/data.txt'})['text'],'safe')
                except (OSError,ValueError):pass
        finally:stop.set();thread.join()
        self.assertEqual((external/'data.txt').read_text(),'PRIVATE')

    def test_listing_text_and_compressed_logs_are_bounded(self):
        for n in range(guest.MAX_ENTRIES+20):(guest.ROOT/f'{n}.txt').touch()
        listing=guest.files({'path':''})
        self.assertLessEqual(len(listing['entries']),guest.MAX_ENTRIES);self.assertTrue(listing['truncated'])
        (guest.ROOT/'large.txt').write_bytes(b'x'*(guest.MAX_TEXT+1))
        with self.assertRaises(ValueError):guest.file_read({'path':'large.txt'})
        with self.assertRaises(ValueError):guest.mutation(self.change('new.txt','x'*(guest.MAX_TEXT+1)),'file_write')
        logs=guest.ROOT/'logs';logs.mkdir()
        (logs/'latest.log').write_text('latest\n')
        with gzip.open(logs/'2026-10-01-1.log.gz','wb') as stream:stream.write(b'archive\n')
        value=guest.logs({'date':'2026-10-01'})
        self.assertIn('2026-10-01',value['dates']);self.assertEqual(value['lines'],['archive'])
        self.assertEqual(guest.logs({})['lines'],['latest'])
        for value in ('2026-02-30','2026-1-01','--help','2026-10-01;id'):
            with self.assertRaises(ValueError):guest.logs({'date':value})
        with gzip.open(logs/'2026-10-02-1.log.gz','wb') as stream:stream.write(b'x\n'*(guest.MAX_LOG_EXPANDED//2+1))
        value=guest.logs({'date':'2026-10-02'})
        self.assertTrue(value['truncated']);self.assertLessEqual(len(value['lines']),200)

    def test_binary_install_replay_verifies_inode_and_bytes(self):
        req=self.request();original=guest.atomic
        def crash(path,value):
            if value['phase']=='committed':raise PowerLoss()
            original(path,value)
        with patch.object(guest,'atomic',crash),self.assertRaises(PowerLoss):guest.install(req)
        self.assertEqual(guest.install(req)['effect'],'committed')
        (guest.ROOT/'server.jar').write_bytes(b'changed')
        with self.assertRaises(ValueError):guest.install(req)

    def test_native_op_is_explicit_verified_and_next_start_only(self):
        guest.CONFIG.parent.mkdir(parents=True);guest.CONFIG.write_text(json.dumps({'software':'paper'}))
        (guest.ROOT/'server.properties').write_text('online-mode=false\nenable-rcon=false\n')
        (guest.ROOT/'config').mkdir()
        (guest.ROOT/'config/paper-global.yml').write_text('proxies:\n  velocity:\n    enabled: true\n    online-mode: true\n    secret: "'+'a'*64+'"\n')
        req=dict(job_id=str(uuid.uuid4()),member=str(uuid.uuid4()),operator=True,identity={'uuid':str(uuid.uuid4()),'name':'Player'})
        original=guest.atomic
        def crash(path,value):
            if value['phase']=='committed':raise PowerLoss()
            original(path,value)
        with patch.object(guest,'atomic',crash),self.assertRaises(PowerLoss):guest.native_operator(req)
        result=guest.native_operator(req)
        self.assertEqual(result['effective'],'next_start');self.assertTrue(result['operator'])
        self.assertEqual(guest.native_operator(req),result)
        self.assertEqual(json.loads((guest.ROOT/'ops.json').read_text())[0]['uuid'],req['identity']['uuid'])
        req={**req,'job_id':str(uuid.uuid4()),'operator':False}
        self.assertFalse(guest.native_operator(req)['operator'])
        self.assertEqual(json.loads((guest.ROOT/'ops.json').read_text()),[])
        (guest.ROOT/'server.properties').write_text('online-mode=false\nenable-rcon=true\n')
        with self.assertRaises(ValueError):guest.native_operator({**req,'job_id':str(uuid.uuid4())})

class StoppedState(unittest.TestCase):
    def test_actual_game_process_state_fails_closed(self):
        for active,sub,pid in [('active','running','123'),('activating','start','0'),('deactivating','stop','123'),('inactive','dead','42')]:
            with patch.object(guest,'systemctl',return_value=SimpleNamespace(stdout=f'ActiveState={active}\nSubState={sub}\nMainPID={pid}\nControlPID=0\nControlGroup=\n')):
                with self.assertRaises(ValueError):guest.server_stopped()
        with patch.object(guest,'systemctl',return_value=SimpleNamespace(stdout='ActiveState=inactive\nSubState=dead\nMainPID=0\nControlPID=0\nControlGroup=\n')):guest.server_stopped()

class AuthProof(unittest.TestCase):
    def test_generated_paper_mapping_and_ambiguous_overrides(self):
        props='#Minecraft server properties\nonline-mode=false\nenable-rcon=false\nmotd=Hello\n'
        paper='_version: 30\nproxies:\n  bungee-cord:\n    online-mode: true\n  velocity:\n    enabled: true\n    online-mode: true\n    secret: '+ 'a'*64+'\nother:\n  key: value\n'
        guest.verify_managed_auth(props,paper)
        for changed in [paper+"'proxies': {}\n",paper+'proxies: {}\n',paper.replace('enabled: true','enabled: "true"'),paper.replace('enabled: true','enabled: false'),paper+'<<: *defaults\n']:
            with self.assertRaises(ValueError):guest.verify_managed_auth(props,changed)
        for changed in [props+'online-mode=true\n',props+' online-mode : false\n',props+'online\\u002dmode=false\n']:
            with self.assertRaises(ValueError):guest.verify_managed_auth(changed,paper)

class AdditionalBoundaries(unittest.TestCase):
    setUp=test_guest.GuestFiles.setUp
    def test_guest_lock_prevents_overlapping_remote_helpers(self):
        with guest.guest_lock():
            with self.assertRaises(ValueError):
                with guest.guest_lock():pass
        with guest.guest_lock():pass

    def test_available_dates_survive_absent_latest_and_large_plain_logs(self):
        logs=guest.ROOT/'logs';logs.mkdir()
        with gzip.open(logs/'2026-10-01-1.log.gz','wb') as f:f.write(b'archive\n')
        value=guest.logs({})
        self.assertFalse(value['available']);self.assertEqual(value['dates'],['2026-10-01'])
        with open(logs/'latest.log','wb') as f:
            f.seek(guest.MAX_LOG_EXPANDED+100);f.write(b'\nlast line\n')
        value=guest.logs({})
        self.assertTrue(value['truncated']);self.assertEqual(value['lines'][-1],'last line')
        self.assertLessEqual(len(json.dumps(value,ensure_ascii=False).encode()),guest.MAX_LOG_BYTES)

    def test_fifo_is_rejected_before_any_read_open(self):
        os.mkfifo(guest.ROOT/'pipe.txt')
        original=guest.os.open
        def check(path,flags,*args,**kwargs):
            self.assertFalse(str(path).startswith('/proc/self/fd/'))
            return original(path,flags,*args,**kwargs)
        with patch.object(guest.os,'open',check),self.assertRaises(ValueError):guest.file_read({'path':'pipe.txt'})

    def test_zip_metadata_is_bounded_before_entry_allocation(self):
        source=self.root/'oversized.zip'
        source.write_bytes(guest.struct.pack('<4s4H2LH',b'PK\x05\x06',0,0,1,1,8*1024*1024+1,0,0))
        with patch.object(guest.zipfile,'ZipFile',side_effect=AssertionError('must not allocate entries')):
            with self.assertRaises(ValueError):guest.extract_world(source,self.root/'destination',1024)
