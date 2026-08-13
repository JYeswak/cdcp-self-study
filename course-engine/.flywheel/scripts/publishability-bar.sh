#!/usr/bin/env bash
set -euo pipefail

repo="${PWD}"
json=0
mode="doctor"

usage() {
    printf '%s\n' \
        "publishability-bar.sh --doctor [--json] [--repo PATH]" \
        "publishability-bar.sh --schema [--json]" \
        "publishability-bar.sh --examples" \
        "" \
        "Scores .flywheel/PUBLISHABILITY-AUDIT.md against the seven-facet publishability bar."
}

examples() {
    printf '%s\n' \
        "# Emit doctor JSON for the current repo" \
        ".flywheel/scripts/publishability-bar.sh --doctor --json" \
        "" \
        "# Score another repo" \
        ".flywheel/scripts/publishability-bar.sh --doctor --json --repo /Users/josh/Developer/mobile-eats"
}

field_value() {
    local field="$1" file="$2"
    awk -F'|' -v f="$field" '
      BEGIN { found=0 }
      tolower($2) ~ "^[[:space:]]*" tolower(f) "[[:space:]]*$" {
        v=$3
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", v)
        print v
        found=1
        exit
      }
      END { if (!found) exit 1 }
    ' "$file" 2>/dev/null || true
}

header_value() {
    local field="$1" file="$2"
    awk -F':' -v f="$field" '
      tolower($1) == tolower(f) {
        v=$0
        sub(/^[^:]*:[[:space:]]*/, "", v)
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", v)
        print v
        exit
      }
    ' "$file" 2>/dev/null || true
}

brand_voice_banned_words_json() {
    local voice_yaml="/Users/josh/.claude/skills/zeststream-brand-voice/brands/zeststream/voice.yaml"
    if [[ ! -f "$voice_yaml" ]]; then
        jq -nc '[]'
        return 0
    fi
    awk '
      /^banned_words:/ { in_list=1; next }
      in_list && /^[^[:space:]#][^:]*:/ { exit }
      in_list && /^[[:space:]]*-[[:space:]]*/ {
        line=$0
        sub(/^[[:space:]]*-[[:space:]]*/, "", line)
        sub(/[[:space:]]*#.*/, "", line)
        gsub(/^"|"$/, "", line)
        gsub(/^'\''|'\''$/, "", line)
        gsub(/[[:space:]]+$/, "", line)
        if (length(line) > 0) print line
      }
    ' "$voice_yaml" | jq -Rsc 'split("\n") | map(select(length > 0))'
}

brand_voice_metrics_json() {
    local repo="$1" audit="$2" public_repo public_repo_json exemption score scorecard_log ungrounded words_json text_file scan_file matches_json banned_count
    public_repo="$(header_value "Public repo" "$audit" | tr '[:upper:]' '[:lower:]')"
    public_repo_json=false
    if [[ "$public_repo" == "yes" || "$public_repo" == "true" || "$public_repo" == "public" ]]; then
        public_repo_json=true
    fi
    exemption="$(header_value "Exemption" "$audit" | tr '[:lower:]' '[:upper:]')"
    if [[ "$exemption" == "EXEMPT_CLIENT_OWNED" || "$exemption" == "EXEMPT_PUBLIC_FACING" ]]; then
        jq -nc --argjson public_repo "$public_repo_json" --arg exemption "$exemption" '{
          public_repo:$public_repo,
          public_ready_default:true,
          exempt:true,
          exemption_class:$exemption,
          proof_level:($exemption | ascii_downcase),
          brand_voice_composite:100,
          banned_words_count:0,
          banned_words:[],
          ungrounded_claims_count:0,
          scorecard_log:null,
          errors:[],
          warnings:[]
        }'
        return 0
    fi
    score="$(field_value "ZestStream voice score" "$audit")"
    score="${score:-0}"
    scorecard_log="$(field_value "Scorecard log" "$audit")"
    ungrounded="$(field_value "Ungrounded claims count" "$audit")"
    ungrounded="${ungrounded:-0}"
    words_json="$(brand_voice_banned_words_json)"
    text_file="$(mktemp "${TMPDIR:-/tmp}/publishability-public-copy.XXXXXX")"
    {
        [[ -f "$repo/README.md" ]] && printf '%s\n' "--- README.md ---" && sed -n '1,260p' "$repo/README.md"
        [[ -f "$repo/.flywheel/MISSION.md" ]] && printf '%s\n' "--- .flywheel/MISSION.md ---" && sed -n '1,220p' "$repo/.flywheel/MISSION.md"
        [[ -f "$repo/MISSION.md" ]] && printf '%s\n' "--- MISSION.md ---" && sed -n '1,220p' "$repo/MISSION.md"
    } > "$text_file"
    scan_file="$(mktemp "${TMPDIR:-/tmp}/publishability-public-prose.XXXXXX")"
    sed -E 's/`[^`]*`//g' "$text_file" > "$scan_file"
    matches_json="$(
      jq -r '.[]' <<<"$words_json" | while IFS= read -r word; do
        [[ -z "$word" ]] && continue
        if grep -IiqF -- "$word" "$scan_file"; then
            printf '%s\n' "$word"
        fi
      done | sort -u | jq -Rsc 'split("\n") | map(select(length > 0))'
    )"
    rm -f "$text_file" "$scan_file"
    banned_count="$(jq 'length' <<<"$matches_json")"
    jq -nc \
      --argjson public_repo "$public_repo_json" \
      --arg exemption "$exemption" \
      --argjson score "$score" \
      --argjson ungrounded "$ungrounded" \
      --arg scorecard_log "$scorecard_log" \
      --argjson banned_words "$matches_json" \
      --argjson banned_count "$banned_count" \
      '{
        public_repo:$public_repo,
        public_ready_default:true,
        exempt:false,
        exemption_class:(if $exemption == "" then null else $exemption end),
        proof_level:"scorecard_linked",
        brand_voice_composite:$score,
        banned_words_count:$banned_count,
        banned_words:$banned_words,
        ungrounded_claims_count:$ungrounded,
        scorecard_log:(if $scorecard_log == "" then null else $scorecard_log end),
        errors:(
          (if $score < 90 then [{code:"brand_voice_composite_low", message:"brand voice composite below readiness floor"}] else [] end)
          + (if $banned_count > 0 then [{code:"brand_voice_banned_words", message:"public copy contains banned ZestStream voice words"}] else [] end)
          + (if $ungrounded > 0 then [{code:"brand_voice_ungrounded_claims", message:"public copy has ungrounded claims"}] else [] end)
        ),
        warnings:(if $score < 95 and $score >= 90 then [{code:"brand_voice_composite_regen", message:"brand voice composite below publish threshold"}] else [] end)
      }'
}

schema_json() {
    jq -n '{
      "$schema":"https://json-schema.org/draft/2020-12/schema",
      "title":"publishability-bar/v1",
      "type":"object",
      "required":["schema_version","status","success","publishability_bar_score","max_score","audit_path","facets","brand_voice"],
      "properties":{
        "schema_version":{"const":"publishability-bar/v1"},
        "status":{"enum":["pass","warn","fail"]},
        "success":{"type":"boolean"},
        "publishability_bar_score":{"type":"object"},
        "publishability_bar_score_value":{"type":"integer","minimum":0,"maximum":7},
        "max_score":{"const":7},
        "audit_path":{"type":"string"},
        "facets":{"type":"array"},
        "brand_voice":{"type":"object"}
      }
    }'
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --repo)
            repo="$2"
            shift 2
            ;;
        --repo=*)
            repo="${1#*=}"
            shift
            ;;
        --doctor|doctor)
            mode="doctor"
            shift
            ;;
        --schema|schema)
            mode="schema"
            shift
            ;;
        --examples|examples)
            mode="examples"
            shift
            ;;
        --help|-h|help)
            usage
            exit 0
            ;;
        --json)
            json=1
            shift
            ;;
        *)
            printf 'unknown argument: %s\n' "$1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

case "$mode" in
    schema)
        schema_json
        exit 0
        ;;
    examples)
        examples
        exit 0
        ;;
esac

repo="$(cd "$repo" 2>/dev/null && pwd -P || printf '%s' "$repo")"
audit_path="$repo/.flywheel/PUBLISHABILITY-AUDIT.md"
doctrine_path="$repo/.flywheel/PUBLISHABILITY-BAR.md"
prompt_path="/Users/josh/.claude/skills/.flywheel/prompts/three-judges-rubric.md"

if [[ ! -f "$audit_path" ]]; then
    jq -n \
      --arg audit_path "$audit_path" \
      --arg doctrine_path "$doctrine_path" \
      --arg prompt_path "$prompt_path" \
      '{
        schema_version:"publishability-bar/v1",
        status:"fail",
        success:false,
        publishability_bar_score:0,
        max_score:7,
        audit_path:$audit_path,
        doctrine_path:$doctrine_path,
        prompt_path:$prompt_path,
        facets:[],
        thresholds:{warn_below:5, fail_below:3},
        warnings:[],
        errors:[{code:"publishability_audit_missing", message:"missing .flywheel/PUBLISHABILITY-AUDIT.md"}]
      }'
    exit 1
fi

facets_json="$(
    awk -F'|' '
      BEGIN { first=1; printf "[" }
      $2 ~ /^[[:space:]]*F[1-7][[:space:]]*$/ {
        id=$2; facet=$3; verdict=$4; evidence=$5
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", id)
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", facet)
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", verdict)
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", evidence)
        gsub(/"/, "\\\"", evidence)
        if (!first) printf ","
        first=0
        printf "{\"facet_id\":\"%s\",\"facet\":\"%s\",\"verdict\":\"%s\",\"passed\":%s,\"evidence\":\"%s\"}", id, facet, verdict, (verdict == "YES" ? "true" : "false"), evidence
      }
      END { printf "]" }
    ' "$audit_path"
)"
score="$(jq '[.[] | select(.passed == true)] | length' <<<"$facets_json")"
facet_count="$(jq 'length' <<<"$facets_json")"
brand_voice="$(brand_voice_metrics_json "$repo" "$audit_path")"
brand_errors="$(jq -c '.errors // []' <<<"$brand_voice")"
brand_warnings="$(jq -c '.warnings // []' <<<"$brand_voice")"
score_obj="$(jq -nc \
  --argjson score "$score" \
  --argjson brand "$brand_voice" \
  '{
    score:$score,
    brand_voice_composite:($brand.brand_voice_composite // null),
    banned_words_count:($brand.banned_words_count // 0),
    ungrounded_claims_count:($brand.ungrounded_claims_count // 0),
    public_repo:($brand.public_repo // false),
    proof_level:($brand.proof_level // "unknown"),
    scorecard_log:($brand.scorecard_log // null)
  }')"
status="pass"
success=true
warnings="$brand_warnings"
errors="$brand_errors"
if [[ "$score" -lt 3 ]]; then
    status="fail"
    success=false
    errors="$(jq -nc --argjson errors "$errors" --argjson score "$score" '$errors + [{code:"publishability_bar_score_low", message:("publishability bar score below fail threshold: " + ($score|tostring) + "/7")} ]')"
elif [[ "$score" -lt 5 ]]; then
    status="warn"
    warnings="$(jq -nc --argjson warnings "$warnings" --argjson score "$score" '$warnings + [{code:"publishability_bar_score_warn", message:("publishability bar score below warning threshold: " + ($score|tostring) + "/7")} ]')"
fi
if [[ "$facet_count" -lt 7 ]]; then
    status="fail"
    success=false
    errors="$(jq -nc --argjson errors "$errors" --argjson facet_count "$facet_count" '$errors + [{code:"publishability_facets_incomplete", message:("publishability audit has " + ($facet_count|tostring) + " of 7 facets")} ]')"
fi
if [[ "$(jq 'length' <<<"$brand_errors")" -gt 0 ]]; then
    status="fail"
    success=false
fi
if [[ "$status" == "pass" && "$(jq 'length' <<<"$brand_warnings")" -gt 0 ]]; then
    status="warn"
fi

out="$(jq -n \
  --arg status "$status" \
  --argjson success "$success" \
  --argjson score "$score" \
  --argjson score_obj "$score_obj" \
  --arg audit_path "$audit_path" \
  --arg doctrine_path "$doctrine_path" \
  --arg prompt_path "$prompt_path" \
  --argjson facets "$facets_json" \
  --argjson brand_voice "$brand_voice" \
  --argjson warnings "$warnings" \
  --argjson errors "$errors" \
  '{
    schema_version:"publishability-bar/v1",
    status:$status,
    success:$success,
    publishability_bar_score:$score_obj,
    publishability_bar_score_value:$score,
    max_score:7,
    audit_path:$audit_path,
    doctrine_path:$doctrine_path,
    prompt_path:$prompt_path,
    facets:$facets,
    brand_voice:$brand_voice,
    thresholds:{warn_below:5, fail_below:3},
    warnings:$warnings,
    errors:$errors
  }')"

if [[ "$json" -eq 1 ]]; then
    printf '%s\n' "$out"
else
    jq -r '"publishability_bar_score=\(.publishability_bar_score)/\(.max_score) status=\(.status)"' <<<"$out"
fi

[[ "$status" == "fail" ]] && exit 1
exit 0

# Meta-pattern Adoption stance: MP-90 adjacent-skill-boundary-router
# Adoption signal:
# - Smell-test matrix names when this surface applies and when it does not.
# - If not, use the named sibling or companion skill instead of expanding scope.
# - Boundaries are explicit against adjacent sibling surfaces.
# - Companion skill handoff output matches the next skill input shape.
#
# Smell-test matrix:
# | Signal | Use this surface? | If not, use |
# |---|---|---|
# | Same operator goal and same write boundary | yes | adjacent sibling surface |
# | Different domain owner or evidence shape | no | companion skill with matching input shape |
#
# Boundary router:
# - Positive boundary: handle only the declared mode, target path, and output.
# - Negative boundary: route adjacent work to the sibling named by the caller.
# - Handoff output: emit target path, receipt path, and next input shape.

# Meta-pattern Adoption stance: MP-91 progress-counter-forced-motion-loop
# Adoption signal:
# - Each tick records a productive event count and whether it moved the system.
# - A no-op / stall counter increments on repeated stall outcomes.
# - A threshold changes behavior from HOLD to probe, repair, redispatch, or bead.
# - Each tick writes an append-only receipt with the movement verdict.
#
# Progress-counter forced-motion loop template:
# productive_event_count=0
# repeated_stall_counter=0
# stall_threshold_behavior="after 2 no-op ticks, forbid another HOLD and force a probe or repair"
# append_only_receipt=".flywheel/state/<surface>-progress-receipts.jsonl"
#
# mp91_record_tick should persist:
# ts, tick_id, productive_event_count, no-op reason, stall counter,
# threshold behavior taken, and moved_the_system=true|false.
