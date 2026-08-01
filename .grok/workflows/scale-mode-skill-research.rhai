// Research bright/dark modes and modulation; map to strudel-rs live-edit recipes.
// Run after project folder trust: /workflow scale-mode-skill-research
// Or validate/run via workflow tool with this script_path / inline script.

let meta = #{
    name: "scale-mode-skill-research",
    description: "Web-research mode bright/dark ladder and modulations; map to strudel-rs scale/.add recipes",
    phases: [
        #{ title: "Research", detail: "parallel theory topics" },
        #{ title: "Map", detail: "normalize to engine modes" },
        #{ title: "Verify", detail: "no unimplemented fence patterns" },
    ],
};

let research_schema = #{
    "type": "object",
    "required": ["summary", "ladder", "sources"],
    "properties": #{
        "summary": #{ "type": "string" },
        "ladder": #{
            "type": "array",
            "maxItems": 12,
            "items": #{ "type": "string" },
        },
        "sources": #{
            "type": "array",
            "maxItems": 8,
            "items": #{ "type": "string" },
        },
        "notes": #{ "type": "string" },
    },
};

let map_schema = #{
    "type": "object",
    "required": ["markdown"],
    "properties": #{
        "markdown": #{ "type": "string" },
        "nl_recipes": #{
            "type": "array",
            "maxItems": 12,
            "items": #{
                "type": "object",
                "required": ["intent", "recipe"],
                "properties": #{
                    "intent": #{ "type": "string" },
                    "recipe": #{ "type": "string" },
                },
            },
        },
    },
};

let verify_schema = #{
    "type": "object",
    "required": ["ok", "issues"],
    "properties": #{
        "ok": #{ "type": "boolean" },
        "issues": #{
            "type": "array",
            "maxItems": 16,
            "items": #{ "type": "string" },
        },
    },
};

phase("Research");

let topics = [
    "parallel diatonic modes ordered brightest to darkest (Lydian to Locrian)",
    "relative major minor and common pop EDM key modulations",
    "dorian vs aeolian brightness; mixolydian vs major character",
];

let jobs = [];
for t in topics {
    let p = "";
    p += "Research this music-theory topic using web_search and open_page/web_fetch. ";
    p += "Topic: ";
    p += t;
    p += ". Return summary, a bright-to-dark mode ladder if relevant (mode names only), ";
    p += "and source URLs. Do not invent modes. Prefer ethanhein, berklee, skoove, or textbooks.";
    jobs.push(#{
        prompt: p,
        label: "research",
        capability_mode: "read-only",
        output_schema: research_schema,
    });
}
let results = parallel(jobs);

let combined = "";
let idx = 0;
for r in results {
    if r != () && r.success && r.output.summary != () {
        combined += "## Topic ";
        combined += idx.to_string();
        combined += "\n";
        combined += r.output.summary;
        combined += "\nLadder: ";
        if r.output.ladder != () {
            for m in r.output.ladder {
                combined += m;
                combined += " > ";
            }
        }
        combined += "\nSources: ";
        if r.output.sources != () {
            for u in r.output.sources {
                combined += u;
                combined += " ";
            }
        }
        combined += "\n\n";
    }
    idx += 1;
}
if combined == "" {
    combined = "No research results; use default ladder: lydian major mixolydian dorian minor phrygian locrian";
}
log("research chars: " + combined.len().to_string());

phase("Map");

let map_prompt = "";
map_prompt += "Map the research below to strudel-rs ONLY.\n";
map_prompt += "Engine modes: major/ionian, minor/aeolian, dorian, phrygian, lydian, mixolydian, locrian, ";
map_prompt += "major:pentatonic, minor:pentatonic, chromatic.\n";
map_prompt += "Allowed live-edit ops: .scale(\"Root:mode\"), .scale(\"<A2:minor D:dorian ...>\"), ";
map_prompt += "scalar .add(n)/.sub(n) (degrees with scale; semitones for named notes), mini @, <> fills, .ply(n).\n";
map_prompt += "FORBIDDEN in examples: stack, .cpm, .lfo, sine.range, chooseCycles, dynamic .lpf(\"<...>\"), sample cp.\n";
map_prompt += "Write markdown for a skill section: bright-dark ladder table, NL recipes (brighter/darker/modulate/transpose), ";
map_prompt += "and 2-3 fenced short setcpm+$: examples.\n";
map_prompt += "Research:\n";
map_prompt += combined;

let mapped = agent(map_prompt, #{
    label: "map-strudel",
    capability_mode: "read-only",
    output_schema: map_schema,
});

if mapped == () || !mapped.success || mapped.output.markdown == () {
    complete(#{ summary: "map failed", markdown: "" });
}

phase("Verify");

let vprompt = "";
vprompt += "Adversarially check this markdown for strudel-rs skill safety.\n";
vprompt += "Set ok=false if any fenced code uses: stack(, .cpm(, .lfo(, sine.range, chooseCycles, ";
vprompt += ".lpf(\"<, sample cp, or pattern-arg .add(\"<.\n";
vprompt += "Scalar .add/.sub/.ply and .scale progression <> are OK.\n";
vprompt += "Markdown:\n";
vprompt += mapped.output.markdown;

let verdict = agent(vprompt, #{
    label: "verify",
    capability_mode: "read-only",
    output_schema: verify_schema,
});

let ok = false;
let issues = [];
if verdict != () && verdict.success && verdict.output.ok != () {
    ok = verdict.output.ok;
    if verdict.output.issues != () {
        issues = verdict.output.issues;
    }
}

let path = write_scratch_file("scale-mode-skill-fragment.md", mapped.output.markdown);
complete(#{
    summary: if ok { "verified scale skill fragment" } else { "fragment needs review" },
    path: path,
    ok: ok,
    issues: issues,
});
