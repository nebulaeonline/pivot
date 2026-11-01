# Pivot.Desktop licensing proposal

Date: 2026-10-31.
Status: **Proposed custom licensing terms; not yet adopted.**

The proposed policy makes Pivot.Desktop free for noncommercial and educational use,
with a paid license for commercial use. Only Desktop changes. Shared components, pivotd and the
REPL remain Apache-2.0. Government administration requires paid licensing, with priority
or advantageous pricing; government-owned education and public research remain eligible
for free use. No price, subscription/perpetual term, activation mechanism or license-key
feature has been authorized or introduced.

## Recommendation

Use a clearly named custom **Pivot Desktop Noncommercial and Education License 1.0**
for the free permission, plus a separately agreed paid commercial license. The proposed
identifier is `LicenseRef-Pivot-Desktop-NC-Education-1.0`, not an SPDX-listed standard.
See the [concrete draft](desktop-license-draft.md). It is a source-available proposal;
do not market the restricted Desktop license as an OSI-approved open-source license.

Unmodified [PolyForm Noncommercial 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0)
is unsuitable for the proposed government-use policy: its organization exemptions include
government institutions regardless of funding. Its education/public-benefit direction
is useful context, but this proposal is not branded as PolyForm or represented as its
standard text. The [publisher's explanation](https://polyformproject.org/why-adopt-polyform)
describes reserving commercial licenses while allowing broad public-benefit use.

The commercial license would grant the relevant commercial/government rights in return
for the agreed fee. The free license alone neither sets a price nor creates an automatic
payment contract. Until a separate agreement exists, uses outside its permissions are
not authorized by that free grant. Commercial sale terms need an actual agreement,
not merely a README saying that payment is expected.

## Component boundary

| Component | Intended license |
| --- | --- |
| clients/dotnet/Pivot.Desktop: covered Desktop-specific source and application | Proposed custom free-use terms or separate paid commercial terms |
| clients/dotnet/Pivot.Avalonia | Apache-2.0 |
| clients/dotnet/Pivot.Client | Apache-2.0 |
| All Rust crates and apps, including pivot-session, pivotd and pivot-cli/REPL | Apache-2.0 |
| Third-party code, fonts and other assets | Their respective existing licenses |
| User workbooks, data, formulas and calculated/exported results | No new content license or ownership claim imposed by Desktop licensing |

The Desktop distribution is a mixed-license package. A Desktop license must not
purport to restrict independently licensed shared components or remove Apache/third-party
notices. Building a separate client against the Apache engine/SDK is not commercial use
of newly restricted Desktop code merely because it uses Pivot's shared components.

## Free and paid uses

| Use | Proposed treatment |
| --- | --- |
| Personal, hobby and other noncommercial activity | Free |
| Genuine teaching, instruction and study, including public/private schools and universities | Free; tuition or paid teaching alone does not disqualify the educational activity |
| Noncommercial scientific/scholarly research for public knowledge | Free, including government-owned public research institutions |
| Nongovernmental charitable/nonprofit public-interest activity | Free when noncommercial; includes public-benefit health, safety and environmental work |
| Business operations, paid professional work, commercial production/consulting or proprietary commercial R&D | Paid commercial agreement |
| Government administration, operations or public-service delivery outside the education/research exemption | Paid agreement with priority/advantageous pricing policy |
| A government-owned school's classroom work or public research project | Free under the same purpose-based exemption |
| Government-owned institution's ordinary finance, payroll, procurement or administrative reporting | Paid government use; educational ownership does not exempt administration |
| A commercial operation run by a charity or university | Paid unless the actual use is independently qualifying instruction or noncommercial public research |

The government pricing preference is a sales policy, not an exemption or promise of a
particular discount or procurement priority. Actual pricing belongs in a written quote
and agreement. Mixed-purpose deployment needs permissions for each actual use; a free
classroom installation does not authorize a separate administrative or business use.
The draft spells out research versus ordinary administrative analysis to avoid granting
a general government exemption through relabeling routine work as research.

## Existing Apache grants and adoption

The current root LICENSE and README describe Apache-2.0 coverage. Apache's
[copyright grant, section 2](https://www.apache.org/licenses/LICENSE-2.0) is perpetual
and irrevocable. Existing Apache-licensed copies and prior recipients' rights cannot
be withdrawn by changing a license file. A commercial user may continue using or
forking the Apache-covered baseline under its terms. A new notice cannot make an
already granted Apache permission require payment.

For a future adoption, record the exact effective release/commit and identify the
Desktop material/new changes over which the licensor can grant the new terms. Preserve
prior Apache rights and third-party/contributor notices. Confirm authority over any
contributions before promising commercial rights; do not label independently owned
material as solely owned by the project. The proposed licensor is Nebulae Foundation,
LLC, matching the current repository notice, subject to confirmation of licensing authority.

Adoption requirements:

1. Adopt a final Desktop-specific license with an explicit effective boundary and
   define the separate commercial offer/contact and pricing terms.
2. Clarify the root license scope and README with the component table; preserve
   Apache coverage for every shared component and both command-line applications.
3. Include the Desktop license and all required Apache/third-party notices in actual
   distributions; add appropriate project/package licensing metadata where applicable.
4. Establish future contributor permissions consistent with offering commercial
   licenses. No historical copyright reassignment is inferred.

No operative LICENSE, README license statement, project metadata, binary packaging or
product behavior has changed in this proposal. The proposed terms require legal review before adoption.
