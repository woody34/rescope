#!/usr/bin/env bash
# seed-users.sh — Insert 100 random users into the running emulator.
# Usage: ./scripts/seed-users.sh [BASE_URL]
#   BASE_URL defaults to http://localhost:4600

set -euo pipefail

BASE="${1:-http://localhost:4600}"
ENDPOINT="$BASE/v1/mgmt/user/create"

FIRST_NAMES=(Alice Bob Charlie Diana Eve Frank Grace Hank Ivy Jack Kate Leo Mia Noah Olivia Pete Quinn Rosa Sam Tina Uma Vic Wendy Xena Yara Zane Aria Blake Cruz Dara Eli Faye Gus Hope Ian Juno Knox Lana Max Nina Owen Pia Reid Sage Theo Ula Vance Wren Xavi Yuki Zara)
LAST_NAMES=(Smith Johnson Williams Brown Jones Garcia Miller Davis Rodriguez Martinez Hernandez Lopez Gonzalez Wilson Anderson Thomas Taylor Moore Jackson Martin Lee Perez Thompson White Harris Sanchez Clark Ramirez Lewis Robinson Walker Hall Allen Young Hernandez King Wright Scott Green Baker Adams Nelson Hill Ramirez Campbell Mitchell)
DOMAINS=(example.com test.dev acme.io demo.org sandbox.net devtools.co emulator.test)

created=0
for i in $(seq 1 100); do
  first="${FIRST_NAMES[$((RANDOM % ${#FIRST_NAMES[@]}))]}"
  last="${LAST_NAMES[$((RANDOM % ${#LAST_NAMES[@]}))]}"
  domain="${DOMAINS[$((RANDOM % ${#DOMAINS[@]}))]}"
  email=$(echo "${first}.${last}.${i}@${domain}" | tr '[:upper:]' '[:lower:]')
  name="${first} ${last}"
  phone="+1$(printf '%010d' $((RANDOM * RANDOM % 10000000000)))"

  # Randomly assign status (90% enabled, 10% disabled)
  status="enabled"
  if (( RANDOM % 10 == 0 )); then status="disabled"; fi

  # Randomly set verified flags
  verified_email=false
  verified_phone=false
  if (( RANDOM % 2 == 0 )); then verified_email=true; fi
  if (( RANDOM % 3 == 0 )); then verified_phone=true; fi

  payload=$(cat <<EOF
{
  "loginId": "$email",
  "email": "$email",
  "name": "$name",
  "givenName": "$first",
  "familyName": "$last",
  "phone": "$phone",
  "verifiedEmail": $verified_email,
  "verifiedPhone": $verified_phone
}
EOF
)

  http_code=$(curl -s -o /dev/null -w '%{http_code}' \
    -X POST "$ENDPOINT" \
    -H 'Content-Type: application/json' \
    -d "$payload")

  if [[ "$http_code" == "200" ]]; then
    created=$((created + 1))
  else
    echo "⚠ Failed to create user #$i ($email) — HTTP $http_code"
  fi
done

echo "✅ Seeded $created / 100 users into $BASE"
