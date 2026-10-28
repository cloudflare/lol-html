# Tree construction tests

These are the html5lib tree construction tests, copied from [html5lib-tests][html5lib-tests] at
revision `9329e64694e7835d0dcff9811e22856ef6ad16f9`, the last one before they were removed from
that repository. They are now maintained in [web-platform-tests][wpt] (WPT).

We only use the inputs (the `#data` sections): the feedback tests compare our tokens against the
ones html5ever produces for the same input. The scripted tests are not included (`scripted/` in
html5lib-tests, `scripted_*.dat` in WPT), since they require script execution.

To replace the files with the ones from WPT revision `$rev`, run this from the root of the
repository (and update the source revision above):

```sh
url=https://raw.githubusercontent.com/web-platform-tests/wpt/$rev/html/syntax/parsing/resources
rm tests/data/tree-construction/*.dat
gh api "repos/web-platform-tests/wpt/contents/html/syntax/parsing/resources?ref=$rev" \
    --jq '.[].name | select(endswith(".dat") and (startswith("scripted_") | not))' |
    while read -r name; do
        curl -sSfLo "tests/data/tree-construction/$name" "$url/$name"
    done
```

[html5lib-tests]: https://github.com/html5lib/html5lib-tests
[wpt]: https://github.com/web-platform-tests/wpt/tree/master/html/syntax/parsing/resources
