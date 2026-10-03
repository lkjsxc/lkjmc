import test from "node:test";
import assert from "node:assert/strict";
import { checkedDatabase } from "./scope.mjs";
const database = "lkjmc_test_protocol_20261003_fixture";
const config = {scope:"isolated-protocol-test",test_database:database,database_url:`postgres://lkjmc:example@127.0.0.1:16543/${database}`};
test("protocol scope admits only explicitly named local test data",()=>assert.equal(checkedDatabase(config),database));
test("shared developer, production, renamed and remote data are refused",()=>{
  for(const change of [{scope:"development"},{test_database:"another"},{database_url:"postgres://lkjmc@127.0.0.1:16543/lkjmc_rebuild"},{database_url:`postgres://lkjmc@database.example:16543/${database}`},{database_url:`postgres://admin@127.0.0.1:16543/${database}`},{database_url:config.database_url+"?host=remote"}])assert.throws(()=>checkedDatabase({...config,...change}));
});
