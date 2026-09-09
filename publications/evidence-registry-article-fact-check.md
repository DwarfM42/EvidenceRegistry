# EvidenceRegistry 記事の事実確認

**ARTICLE FACT-CHECK / NON-AUTHORITATIVE SUPPORTING MATERIAL**

## 対象と基準点

| 項目 | 値 |
|---|---|
| 記事タイトル | AI Agentの出力は保存できる。でも、何を証拠として受け入れた？ |
| 記事識別子 | `ER-PEP-ARTICLE-2026-09-09`（この資料での識別子） |
| 確認対象の記事原稿 | UTF-8 Markdown、19,141 bytes、SHA-256 `89a2d6bd2cefb4c9c4e436e9b80f0917315fb39b22d0813fe29f85d80018d7a6` |
| 記事時の事実確認日 | 2026-09-09（日本標準時） |
| 本資料の整理日 | 2026-09-09（日本標準時） |
| EvidenceRegistry commit | `0d9e82523a5b0ff9b6d10710a5f643ac3bf6061e` |
| EvidenceRegistry tree | `592d0cdfe0b230f540afcd5ef0964a9be7162ad2` |
| 対応するrelease | [v0.1.0 — source release](https://github.com/DwarfM42/EvidenceRegistry/releases/tag/v0.1.0) |
| annotated tag object | `df21f2deedc23e0bd1f357ab9135c8f2c32533fe`（署名済みtagという主張ではない） |

記事はAIの支援を受けて作成されました。執筆時には、本文の生成とは別の調査・レビューで、実装・形式モデル・保存済み検証結果を固定したsource identityに照合しました。この資料は、そのうち **EvidenceRegistryに関する主張** の公開用整理です。「独立」は別の調査・レビュー作業という意味で、人間の第三者認証機関による認証や、AIから独立した保証ではありません。

記事はPerformanceEvidenceProbeも扱いますが、その実装・測定結果の全面的な再監査は本資料の対象外です。執筆動機や将来の設計目標も、repositoryから歴史的事実として証明したものではありません。記事の公開URLはここでは確定していません。上記digestは確認した原稿を識別するもので、その原稿の公開・第三者による取得可能性を証明しません。

**この資料、記事、その執筆・レビューは、EvidenceRegistry authority、採用決定、Formal Verification Record、製品qualificationのいずれでもありません。**

## 判定区分

- **Source直接確認**：固定commitの実装・モデル・文書に直接存在する内容。
- **保持実行証拠**：保存された実行結果・検証記録に支えられる内容。新しい実行とは区別する。
- **部分支持／証拠不足**：一部は確認できるが、より強い主張に必要なidentity bindingや原本が不足する内容。
- **意図的非再実行**：記事や本資料の見栄えを強くする目的で、完了済み検証を再実行していない事項。
- **歴史的provenance gap**：過去の実行・採用と、後から読める保持物との結び付けが不完全な事項。これだけで現在の製品欠陥や、過去に決定がなかったことを意味しない。

## Claim table

| ID | 記事の主張・確認項目 | 判定 | 確認できた範囲／残る境界 | 根拠 |
|---|---|---|---|---|
| A01 | Rust libraryとread-only JSON CLIがある | Source直接確認 | CLIのcommandは`journal verify`。libraryの全機能をCLIが公開するわけではない。一般的なevidence収集・DBサービスではない。 | [S1], [S2] |
| A02 | Record framingとexact-byte identityを検査する | Source直接確認 | `StrictRecordFrame`によるenvelope検査と、別のtyped decoderによる対応schemaのbody検査を分ける。どちらも自動的なreference resolutionやauthorityではない。 | [S1], [S3] |
| A03 | retained Journalをreplayする | Source直接確認 | caller指定順序のentries、hash/reference、実装済みstate-only遷移を扱う。Journal-only replayは参照先Recordの内容・Policy satisfactionを確立しない。 | [S1], [S2], [S3] |
| A04 | positive Freeze authorityと成功するterminal Review Admission publicationは到達不能 | Source直接確認 | 固定sourceのsemantic gatesは未確立authorityを推測で補わない。型名・terminal処理・成功enumの存在はpublic routeの成功到達性ではない。 | [S1], [S4], [S5] |
| A05 | preterminal failureは完了したPolicyの不合格・不確定ではない | Source直接確認 | 個別evaluator、completed Policy、Admission disposition、Record構築、Journal appendを別に扱う。`UNAVAILABLE`を満足や成功に読み替えない。 | [S1], [S4], [S5] |
| A06 | 記事時のsynthetic Genesis CLI例はexit 0でもauthority/admissionがUNAVAILABLE | 保持実行証拠 | 保存されたreceiptは実exit 0を記録し、保持stdoutは`JOURNAL_ONLY_REPLAY` / `VALID` / `UNAVAILABLE` / `UNAVAILABLE`。参照未解決のsynthetic入力であり、実Registry authorityを確立した実験ではない。raw出力とreceiptの識別は下記[E1]。 | [S1], [E1] |
| A07 | Lifecycleの小さいsymbolic/state-onlyモデルがある | Source直接確認＋保持実行証拠 | event/object/state分類、terminality、状態限定の遷移法則。保持logは`20 verified, 0 errors`。これはlemma数ではない。Journal bytes、payload、authority、Rust実装全体の証明ではない。 | [S6], [S7], [S8] |
| A08 | SupportImpactの二項maxとterminal floorの性質を扱う | Source直接確認＋部分支持 | 明示rank、交換・結合・冪等・単位元、二要素の順序不変、抽象floorの非復活。v2の保持結果は`11 verified, 0 errors`だがsource/log digest bindingが欠ける。任意長sequenceのfold/permutationや実Journal時系列の検証とは書けない。 | [S9], [S10], [S11] |
| A09 | LifecycleとSupportImpactの実行provenanceは同じ強さではない | 歴史的provenance gap | Lifecycleのsource/log SHA-256は保持bytesと一致。SupportImpact v1/v2には両digest fieldがなく、同等の実行bindingを再確立できなかった。現在hashを計算しても過去の欠落は埋まらない。 | [S7], [S10], [S12] |
| A10 | 未証明frontierがある | Source直接確認＋証拠不足 | `TerminalPredecessorCanOnlyBeObserved`はresource frontierとして記録され、現モデルに含まれない。失敗時のexact source/log一式は固定treeに同梱されず、説明記録を失敗実行の新規再現とは扱わない。仕様gapやruntime defectとの断定もしない。 | [S6], [S13] |
| A11 | Lifecycle v0.10.4／Cross-Reference v0.5の採用記録がある | Source直接確認＋部分支持 | detached recordはexact candidate commit/tree、対象blob/digest/size、Owner adoption、両reviewのclean完了を記録する。対象文書identityは確認できるが、tree外のreview原本を記事時に再認証したわけではない。 | [S14] |
| A12 | 一部の後続仕様の採用chainは十分に再確立できない | 歴史的provenance gap | v0.10.7はv0.10.6をOwner-frozen inputと記すが、固定treeにはv0.10.6について同等のdetached adoption chainがない。「採用されなかった」「番号最大の文書がauthority」という結論にはしない。 | [S14], [S15], [S16] |
| A13 | 記事確認のためにDafnyを新しく実行した | 不支持／非再実行 | 記事時はsourceと保持provenance・結果を読んだ。許可された探索範囲では使える既存toolを発見できず、toolの取得・install・Dafny実行を記事のためには行っていない。全machine上の不存在証明ではない。 | 記事時の調査記録；[S7], [S10]は保持結果のみ |
| A14 | Rust・Dafny・記事レビューにより製品全体の正しさや受理権限が確立する | 不支持 | 型、hash、model proof、テスト、AIの説明はそれぞれの範囲の証拠。形式モデルとRust/vectorの全面的な対応証明はなく、対応registryにも未接続項目が残る。 | [S1], [S6], [S9], [S17] |

## 保存結果のidentityと公開上の限界

### E1 — 記事時のCLI観測

以下は記事時に保持されたbytesを本資料作成時に再計算した識別子です。新規CLI実行ではありません。元の実行receiptはbinaryとinputのdigest、引数、exitを記録していますが、ここに列挙するstdout/receipt digestは本資料作成時の結び付けです。実行時にすべてのhashが一つのreceiptへ記録されていた、という主張ではありません。

| 保持物の役割 | bytes | SHA-256 |
|---|---:|---|
| raw stdout | 309 | `fd4ff535f8251386089a79e8cf9f62fbb42afbf0e8920a5d9a9479320cb5df47` |
| raw stderr（空） | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| 実行receipt | 484 | `ea90caf83b5bc13d65ef8b4684ae3bca67428f59bb4ea34ac4d7d10732b87792` |

これらのraw記録は本資料に同梱・公開していません。したがって読者が公開sourceだけから当該の過去実行を独立に再認証できるとは主張しません。hashの列挙は、原本の可用性、収集時刻、custody、実際にchildが読んだ瞬間のbytes、署名による真正性を証明しません。

### 公開tree内のformal evidence

| 対象 | 確認したSHA-256 | 位置づけ |
|---|---|---|
| Lifecycle source | `ab660efd5b2b944653880e1d6400de0767c88e804e2a8949f40dde9043216624` | provenance記載digestと一致 |
| Lifecycle retained log | `eda707aab4b15e8472f7d1722c5ad2994e3afb7a0b8d69ae37caae535a0af717` | provenance記載digestと一致。provenanceはCRLFからLFのみの変換を宣言 |
| SupportImpact source | `52480cf113a63e832858543f574b7d5c0f628575d6441f97e87038adc0b5f3ca` | 今回読めたsourceのidentity。過去runへの新しいbindingではない |
| SupportImpact v2 retained log | `9b7129a137e59db9c540aea34d816ae3f8f2669487847f440bb678a24a0cb82c` | COMMAND/RESULT wrapper。元provenanceにlog digestなし。raw process exitの証拠とは同一視しない |

Lifecycleのdigest一致も、当時のtoolの全依存物・実行環境・実際のloadを全面的に再認証したことにはなりません。どちらのprovenanceも自身をEvidenceRegistry Formal Verification Recordではないと明記しています。

## 実施しなかった検証と、後続保守との分離

- **記事時にDafnyは新規実行していません。** Lifecycleのdeferred theoremを再試行したり、保持失敗logを成功で置き換えたりしていません。
- 記事時にはWindowsでRust tests/buildとsynthetic CLIを実行した記録があります。これは「記事では何も実行していない」という意味ではありません。一方、本公開資料を整えるためにそれらを再実行したり、完了済みv0.1.0の三OS qualification・Skill検証をやり直したりしていません。
- tree外のhistorical review原本を記事時に再認証していません。文書にあるreviewの報告と、raw原本の再認証は別です。
- 後続仕様について、過去に存在したはずのOwner decisionやreviewを推定・再作成していません。embedded Statusだけでも、文書の版番号だけでも採否は決まりません。
- 後からSupportImpactのprovenance保守で新しいDafny runを行う場合、そのsource/tool/invocation/log/resultの新しいbindingは**その新しいrunだけ**のものです。記事時のA08/A09/A13の結果を修正せず、歴史的gapを残します。
- 将来向けのadoption-process改善も、この資料も、過去の採用記録を補完したりfrozen authorityを変更したりしません。

## Sources（すべてEvidenceRegistryの固定commit）

[S1]: https://github.com/DwarfM42/EvidenceRegistry/blob/0d9e82523a5b0ff9b6d10710a5f643ac3bf6061e/README.md
[S2]: https://github.com/DwarfM42/EvidenceRegistry/blob/0d9e82523a5b0ff9b6d10710a5f643ac3bf6061e/src/main.rs
[S3]: https://github.com/DwarfM42/EvidenceRegistry/blob/0d9e82523a5b0ff9b6d10710a5f643ac3bf6061e/src/lib.rs
[S4]: https://github.com/DwarfM42/EvidenceRegistry/blob/0d9e82523a5b0ff9b6d10710a5f643ac3bf6061e/src/authoritative_store.rs
[S5]: https://github.com/DwarfM42/EvidenceRegistry/blob/0d9e82523a5b0ff9b6d10710a5f643ac3bf6061e/tests/review_admission_runtime.rs
[S6]: https://github.com/DwarfM42/EvidenceRegistry/blob/0d9e82523a5b0ff9b6d10710a5f643ac3bf6061e/formal/Lifecycle.dfy
[S7]: https://github.com/DwarfM42/EvidenceRegistry/blob/0d9e82523a5b0ff9b6d10710a5f643ac3bf6061e/formal/Lifecycle-tool-provenance-v1.json
[S8]: https://github.com/DwarfM42/EvidenceRegistry/blob/0d9e82523a5b0ff9b6d10710a5f643ac3bf6061e/formal/Lifecycle-verification-v1.log
[S9]: https://github.com/DwarfM42/EvidenceRegistry/blob/0d9e82523a5b0ff9b6d10710a5f643ac3bf6061e/formal/SupportImpact.dfy
[S10]: https://github.com/DwarfM42/EvidenceRegistry/blob/0d9e82523a5b0ff9b6d10710a5f643ac3bf6061e/formal/SupportImpact-tool-provenance-v2.json
[S11]: https://github.com/DwarfM42/EvidenceRegistry/blob/0d9e82523a5b0ff9b6d10710a5f643ac3bf6061e/formal/SupportImpact-verification-v2.log
[S12]: https://github.com/DwarfM42/EvidenceRegistry/blob/0d9e82523a5b0ff9b6d10710a5f643ac3bf6061e/formal/SupportImpact-tool-provenance-v1.json
[S13]: https://github.com/DwarfM42/EvidenceRegistry/blob/0d9e82523a5b0ff9b6d10710a5f643ac3bf6061e/formal/Lifecycle-formal-frontier-v1.md
[S14]: https://github.com/DwarfM42/EvidenceRegistry/blob/0d9e82523a5b0ff9b6d10710a5f643ac3bf6061e/docs/FREEZE-RECORD-LIFECYCLE-v0.10.4-CROSS-REFERENCE-v0.5.md
[S15]: https://github.com/DwarfM42/EvidenceRegistry/blob/0d9e82523a5b0ff9b6d10710a5f643ac3bf6061e/docs/EVIDENCE-REGISTRY-LIFECYCLE-SPEC-v0.10.6.md
[S16]: https://github.com/DwarfM42/EvidenceRegistry/blob/0d9e82523a5b0ff9b6d10710a5f643ac3bf6061e/docs/EVIDENCE-REGISTRY-LIFECYCLE-SPEC-v0.10.7.md
[S17]: https://github.com/DwarfM42/EvidenceRegistry/blob/0d9e82523a5b0ff9b6d10710a5f643ac3bf6061e/formal/property-vector-correspondence.json
[E1]: #e1--記事時のcli観測

番号リンクは上記commitのbytesに固定しています。リンク先の歴史的文書には当時の非公開原本を指す参照が残る場合があります。本資料はそれを公開原本の可用性として扱わず、非公開のlocatorやtraceを転載しません。

ナビゲーション：[repository README](../README.md)（移動先の最新文書であり、本fact-checkの固定sourceを置き換えません）。

**最終境界：`STRUCTURALLY_VALID != AUTHORITATIVELY_VALID`。記事を読む、書く、レビューする、hashを再計算することは、それ自体ではEvidenceRegistry authorityを成立させません。**
