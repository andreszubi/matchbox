# ColdFusion (CFML)

MatchBox can run ColdFusion source: `.cfm` tag pages, `.cfc` components and `.cfs` scripts. CF code is parsed into the same syntax tree the BoxLang parser produces, so MatchBox's compiler and VM run it unchanged. This follows the design of BoxLang on the JVM, which also turns CF syntax into BoxLang.

## Which Files Are CF

The file extension picks the parser:

| Extension | Read as |
| :--- | :--- |
| `.cfm`, `.cfml` | CF tag template (`<cfset>`, `<cfoutput>`, …) |
| `.cfc`, `.cfs` | CFScript |
| `.bxm` | BoxLang template |
| `.bx`, `.bxs`, anything else | BoxLang script |

This applies everywhere MatchBox reads a source file: the CLI (`matchbox page.cfm`), `include`, class imports (`import models.User;` finds `models/User.cfc` after `.bxs` and `.bx`), the webroot server (`.cfm` pages and `index.cfm`), and app server templates.

## Runnable Example

Save this as `hello.cfm`:

```html
<cfset name = "World">
<cfset count = 3>
<cfoutput>Hello, #name#!</cfoutput>
<cfif count GT 2>count is big<cfelse>count is small</cfif>
<cfscript>
    function shout( text ) {
        return uCase( text ) & "!";
    }
    writeOutput( shout( "cf runs on matchbox" ) );
</cfscript>
```

From the MatchBox project root, run it:

```bash
cargo run -- hello.cfm
```

After two blank lines (the line breaks after the two `<cfset>` tags are output as text, as in other CF engines), it prints:

```text
Hello, World!
count is big
CF RUNS ON MATCHBOX!
```

To serve CF pages, point the web server at a folder that contains `.cfm` files. `index.cfm` is served for a directory:

```bash
cargo run -p matchbox_server -- --port 8080 --webroot my-site
```

## What Is Supported

| CF construct | Status |
| :--- | :--- |
| `<cfset>`, `<cfoutput>`, `<cfif>` / `<cfelseif>` / `<cfelse>`, `<cfscript>`, `<!--- comments --->` | ✅ Works, same as the `<bx:…>` tags |
| Tag names in any case (`<CFSET>`, `<CfOutput>`) | ✅ Works |
| `<cfloop>`, `<cftry>` / `<cfcatch>`, `<cfswitch>`, `<cfinclude>` | ⚠️ Parse the same as `<bx:loop>`, `<bx:try>`, `<bx:switch>`, `<bx:include>`, so they also have the same bugs those BoxLang tags have today (see below) |
| `component { … }` in `.cfc` / `.cfs` | ✅ Works, read as `class { … }` |
| `NOT x`, `a MOD b` | ✅ Works, same as `!x` and `a % b` (in BoxLang code too) |
| `GT`, `EQ`, `NEQ`, `GTE`, `LTE`, `IS`, `IS NOT`, `CONTAINS`, `AND`, `OR`, `EQV` | ✅ Works |
| `var`, typed parameters, `try` / `catch` in CFScript | ✅ Works |
| Function attributes after the parameter list: `function f() output=false { }` | ✅ Parses, stored like the annotation `@output(false)` (CF `output=` behavior is not applied) |
| Struct literals with `=`: `{ a = 1 }` | ✅ Works, same as `{ a : 1 }` |
| CF-only functions: `valueArray()`, `queryGetRow()`, `getTemplatePath()` | ✅ Renamed to `queryColumnData()`, `queryRowData()`, `getBaseTemplatePath()` (the query functions need the `bif-datasource` build feature) |
| CF-only names: `cfcatch`, `cfthread`, `cffile`, `cfftp`, `cfhttp`, `cfquery`, `cfdocument`, `cfstoredproc`, `thistag` | ✅ Renamed to `bxcatch`, `bxthread`, …, `thiscomponent`. Struct keys, member names, named-argument names and user functions with these names are left alone |

### Known Template Bugs

These come from MatchBox's existing `.bxm` template runtime, not from the CF parser. A `.cfm` page uses the same runtime, so it shows them too:

- `<cfloop>` with `from`/`to` or `array` never stops.
- A loop or `<cfif>` inside `<cfoutput>` prints the wrong result.
- `<cfoutput>` inside a `<cfif>` block stops everything after it from printing.
- `<cftry>` / `<cfcatch>` prints nothing.
- `<cfswitch>` always runs `<cfdefaultcase>`.
- `<cfinclude template="…">` reports "Include file not found" with an empty path. Use `include "file.cfm";` inside `<cfscript>` instead.

## Not Supported

- Tags inside script, such as `cfhttp( url = "…" ) { }`
- `<cfquery>` and SQL
- Adobe ColdFusion vs. Lucee engine differences
- Upper-casing struct keys the way some CF engines do
- CF `output=` semantics on functions and components
- CF custom tags (`<cfx_mytag>`, `<cf_mytag>`): they stop with the parse error "CF custom tags are not supported"
- Tag-based components (`<cfcomponent>` in a `.cfc`): they stop with a parse error. Write components in script syntax, `component { … }`
- `.cfm` / `.cfs` routes in [ESP32 embedded apps](building-and-deploying/esp32.md), which accept only `.bxm` and `.bxs`
