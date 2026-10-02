# 配置の正本

本番インフラの定義と反映処理は https://forgejo.lkjsxc.com/lkjsxc/gitops の
`infra/lkjmc_platform`、`services/lkjmc`、`ops/gitops-reconcile.py` で管理する。
CI合格後の main をホストが取得し、正本state・保存plan・単一writer lockを
照合して反映する。データ削除・リソース置換は自動反映しない。

ファイル・ディレクトリには役割を表す名前を使い、`v2`、`new`、`rebuild`などの
世代・作業状態を示す接尾辞を付けない。正本stateは`lkjmc_platform.tfstate`。
既存データの識別子と過去の配備記録は、その参照関係を維持する。

このディレクトリの `inventory.json` はアプリのネットワーク契約の試験用。
インフラを作成するスクリプトやstateは含まない。agentは本番で実際のVM、
NIC、ACL、資源量、root管理の配備証跡も検査し、契約と異なる場合は操作しない。
旧生成スクリプトの控えは開発環境の `.local/pre-gitops-infrastructure-source`
に保存した。そこから本番へ反映しない。

agentはGitOpsと同じ既存 `operations.lock` を使用する。設定・鍵・正本stateは
root専用とし、CI・ゲストへ渡さない。動的VM操作ごとに管理コミット、対象、
要求、実VM状態を保存し、完了時にplanハッシュを再検査する。

容量はCore上の予約と実VMを重ねて数える。個人VMのCPU・RAM・ディスクの総量、
実測したホスト空きRAM、poolの160GiB以上の余裕、agent保存領域の上限を確認する。
ログ取得や配置のための一時起動、公式SMPの再開にも空き容量の確認を行う。
アーカイブ上限は明示設定し、Incus exportの出力をOSのファイルサイズ制限で止める。
上限に達した保存を成功と扱わず、完成済みの世代を勝手に削除しない。
