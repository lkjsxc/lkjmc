# lkjmc

新しい lkjmc の独立した実装。既存のコード、DB、ワールドを取り込まない。

Rust API / worker、PostgreSQL、React Web、Velocity / Paper adapters、Incus host agent で構成する。
進捗と実測の証拠は [実装記録](docs/implementation.md)、公開条件は [受入条件](docs/acceptance.md) に記録する。

このリポジトリの作成だけでは本番公開済みを意味しない。実際の動作と検証結果で公開を判断する。

ローカル検証には Rust、Node、JDK25、PostgreSQL18、同じ版の `pg_dump` / `pg_restore` が必要。
`python3 scripts/dev.py test` は独立DBで統合試験を実行する。復元試験用の新規DBも作成・削除するため、開発DB専用の資格情報を使う。
`LKJMC_PG_DUMP` に PostgreSQL18 の `pg_dump` の絶対パスを指定すると、同じディレクトリの `pg_restore` で保存形式を検査する。`scripts/dev.py` は `.local/pg-client/root/usr/lib/postgresql/18/bin/pg_dump` があれば自動的に使用する。
ゲスト内ファイル処理だけの検証は `python3 -m unittest discover -s tests/guest -v`。
