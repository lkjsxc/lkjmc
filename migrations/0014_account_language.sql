-- English is explicit; browser and Minecraft client locales do not opt users in.
ALTER TABLE accounts ADD COLUMN language text NOT NULL DEFAULT 'en';
ALTER TABLE accounts ADD CONSTRAINT account_language_code CHECK (language ~ '^[a-z]{2,3}(-[A-Za-z0-9]{2,8})*$');
