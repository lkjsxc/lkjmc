CREATE UNIQUE INDEX one_land_escrow_per_claim ON assets(claim_id)
    WHERE kind='land' AND state IN ('capturing','escrowed','listed','placing','quarantined');
