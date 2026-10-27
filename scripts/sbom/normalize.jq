# Combine observations of one component without losing array-valued metadata.
# A package required by any root remains required in the combined inventory.
def merge_component:
  reduce .[] as $component ({};
    reduce ($component | keys_unsorted[]) as $key (.;
      .[$key] as $left | $component[$key] as $right
      | .[$key] = (
          if $left == null then $right
          elif $right == null or $left == $right then $left
          elif $key == "scope" then
            ["required", "optional", "excluded"] as $scopes
            | [$left, $right] | min_by(. as $scope | $scopes | index($scope))
          elif ($left | type) == "object" and ($right | type) == "object" then
            [$left, $right] | merge_component
          elif ($left | type) == "array" and ($right | type) == "array" then
            if $key == "components" then
              ($left + $right) | group_by(."bom-ref")
              | map(if .[0]."bom-ref" == null then .[] else merge_component end)
            else ($left + $right) | unique end
          else error("Conflicting component field: \($key)")
          end
        )
    )
  );

def normalize:
  # References are local to an input BOM. Rewrite them before merging, since
  # Cargo can use the same ref with different relative-path package URLs.
  (reduce (.. | objects | select(."bom-ref"? != null and .purl? != null)) as $component
    ({};
      $component."bom-ref" as $ref
      | if has($ref) and .[$ref] != $component.purl then
        error("Ambiguous component reference: \($ref)")
      else .[$ref] = $component.purl end
    )) as $refs
  | walk(if type == "object" then
      (if has("bom-ref") then ."bom-ref" = ($refs[."bom-ref"] // ."bom-ref") else . end)
      | (if has("ref") then .ref = ($refs[.ref] // .ref) else . end)
      | (if has("dependsOn") then .dependsOn |= map($refs[.] // .) | .dependsOn |= unique else . end)
    else . end)
  # The flat CLI merge retains observations with differing properties or scope.
  # Coalesce those observations, including nested Cargo targets/Gradle modules.
  | walk(if type == "object" and has("components") then
      .components |= (group_by(."bom-ref") | map(if .[0]."bom-ref" == null then .[] else merge_component end))
    else . end)
  # cdxgen can also repeat the project itself in the top-level component list.
  | .metadata.component."bom-ref" as $root
  | if $root != null then
      .metadata.component = ([.metadata.component, (.components[]? | select(."bom-ref" == $root))] | merge_component)
      | .components |= map(select(."bom-ref" != $root))
    else . end
  | if has("dependencies") then .dependencies |= (group_by(.ref) | map(merge_component)) else . end;
