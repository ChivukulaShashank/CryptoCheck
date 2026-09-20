-- Configuration script for PQC timing harness
config = {
    algorithm = "ML-DSA-44",
    iterations = 5000,
    output_file = "timing_traces.csv"
}

print("[Lua] Configuration loaded: Running " .. config.algorithm .. " for " .. config.iterations .. " iterations.")
return config.iterations