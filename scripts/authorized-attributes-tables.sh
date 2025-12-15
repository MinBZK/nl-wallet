#!/bin/bash

function print_usage() {
    local program="$(basename $0)"
    echo "Usage: $program [<.json>...]"
    echo ""
    echo "Format VCT claims from JSON definition document(s) as markdown tables."
    echo "Outputs to stdout. For example:"
    echo ""
    echo "    $program devenv/eudi:{pid,pid-address}:*.json"
}

# output claim properties in sorted tsv format, extends-aware
# shellcheck disable=SC2016 # not expanding in filter and claims is intentional
function json_claims_to_tsv() {
    local file="$1"
    local parent="$2"

    local header='(["Claim Path","Label","Description","Language"],'

    local filter='| (.path|join(".")) as $path
                  | .display[] 
                  | [$path, .label, .description, .lang]]
                  | sort_by([.[0], .[3]])
                  | .[])) 
                  | @tsv'

    if [[ $parent ]]; then
        claims='([($parent[0].claims + .claims)[] '
        jq -r --slurpfile parent "$parent" "$header$claims$filter" "$file"
    else
        claims='([.claims[] '
        jq -r "$header$claims$filter" "$file"
    fi
}

# output a given tsv as a markdown table
function tsv_to_markdown_table() {
    local tsv="$1"
    echo "$tsv" | awk '
        BEGIN {
            FS="\t"
        }

        {
            n=(NF>n?NF:n);

            for(i=1;i<=NF;i++) {
                if(length($i)>w[i])
                    w[i]=length($i)
            }
            rows[NR]=$0
        }

        END {
            for(r=1;r<=NR;r++) {
                split(rows[r],f,FS);
                printf("|");

                for(i=1;i<=n;i++) {
                    printf(" %-*s |", w[i], f[i])
                }
                printf("\n");
                if(r==1) {
                    printf("|");
                    for(i=1;i<=n;i++) {
                        dash="";
                        for(j=1;j<=w[i];j++)
                            dash=dash"-";
                        printf(" %s |", dash)
                    }
                    printf("\n")
                }
            }
        }'
}

# print help if no arguments given
if [ -z "$1" ]; then
    print_usage
    exit 1
fi

# print help if help requested, set files array
case $1 in
    -h|--help|help)
    print_usage
    exit 1
    ;;
    *)
    files=("$@")
    ;;
esac

# iterate over VCT json documents, print header, generate tables
for file in "${files[@]}"; do
    extends=$(jq -r '.extends // empty' "$file")

    printf "\n### Claims in %s)\n\n" "$(basename "${file/.json/}")"

    if [ -n "$extends" ]; then
        parent="$(dirname "$file")/${extends#urn:}.json"
        tsv_to_markdown_table "$(json_claims_to_tsv "$file" "$parent")"
    else
        tsv_to_markdown_table "$(json_claims_to_tsv "$file")"
    fi
done
