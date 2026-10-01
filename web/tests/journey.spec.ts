import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
const session=JSON.parse(readFileSync('../.local/browser-session.json','utf8'));
test.beforeEach(async({context})=>{await context.addCookies([{name:'lkjmc_session',value:session.token,url:'http://127.0.0.1:18091',httpOnly:true,sameSite:'Lax'}]);});

test('desktop pages load real API states and retain working navigation',async({page})=>{
  const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
  await page.goto('/');await expect(page.getByRole('heading',{name:'ホーム',exact:true})).toBeVisible();
  for(const name of ['遊ぶ','つながり','暮らし','マーケット','冒険','マイサーバー','設定','運営']){
    await page.getByRole('navigation').getByRole('link',{name,exact:true}).click();
    await expect(page.getByRole('heading',{name,exact:true})).toBeVisible();
    await expect(page.getByText('読み込んでいます…',{exact:true})).not.toBeVisible();
    await expect(page.locator('[role=alert]')).toHaveCount(0);
  }
  await page.getByRole('navigation').getByRole('link',{name:'ホーム',exact:true}).click();
  await page.screenshot({path:'../.local/home-desktop.png',fullPage:true});
  expect(errors).toEqual([]);
});

test('a group and a message persist through a browser reload',async({page})=>{
  const name=`集会所 ${Date.now()}`;
  await page.goto('/#social');await expect(page.getByRole('heading',{name:'会話',exact:true})).toBeVisible();
  await page.getByRole('heading',{name:'会話',exact:true}).locator('..').getByRole('button').click();
  await page.getByLabel('グループ名',{exact:true}).fill(name);await page.getByRole('dialog').getByRole('button',{name:'作成する',exact:true}).click();
  await expect(page.getByRole('dialog')).toHaveCount(0);
  await page.getByRole('button',{name:new RegExp(name)}).click();
  await page.getByLabel('メッセージ',{exact:true}).fill('これは実際のデータベースに保存されるメッセージです。');
  await page.getByRole('button',{name:'送信',exact:true}).click();
  await expect(page.getByText('これは実際のデータベースに保存されるメッセージです。',{exact:true})).toBeVisible();
  await page.reload();await page.getByRole('button',{name:new RegExp(name)}).click();
  await expect(page.getByText('これは実際のデータベースに保存されるメッセージです。',{exact:true})).toBeVisible();
  await page.getByRole('button',{name:'通報',exact:true}).click();
  await page.getByRole('checkbox').check();await page.getByRole('button',{name:'提出内容を確認',exact:true}).click();
  await expect(page.getByRole('dialog').getByText('これは実際のデータベースに保存されるメッセージです。',{exact:true})).toBeVisible();
  await page.getByLabel('通報の理由',{exact:true}).fill('ブラウザ受入検証');
  await page.getByRole('button',{name:'この内容で通報する',exact:true}).click();await expect(page.getByRole('dialog')).toHaveCount(0);
  await page.goto('/#settings');await expect(page.getByText('ブラウザ受入検証',{exact:true})).toBeVisible();
});

test('mobile navigation and dialogs fit a narrow viewport',async({page})=>{
  await page.setViewportSize({width:390,height:844});await page.goto('/');
  await expect(page.getByRole('heading',{name:'ホーム',exact:true})).toBeVisible();
  await page.getByRole('button',{name:'メニューを開く'}).click();
  await page.getByRole('navigation').getByRole('link',{name:'暮らし',exact:true}).click();
  await page.getByRole('button',{name:'土地を保護',exact:true}).click();
  await expect(page.getByRole('dialog')).toBeVisible();
  const width=await page.evaluate(()=>({scroll:document.documentElement.scrollWidth,client:document.documentElement.clientWidth}));
  expect(width.scroll).toBeLessThanOrEqual(width.client);
  await page.screenshot({path:'../.local/land-mobile.png',fullPage:true});
});
