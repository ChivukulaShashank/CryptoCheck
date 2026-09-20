settings = {
    -- Switch modes: "train" builds the model weights; "test" runs evaluation / cracking simulation
    mode = "test", 
    
    -- Random Forest Hyperparameters
    num_trees = 10,       -- Number of decision trees in the ensemble
    max_depth = 4,        -- Maximum depth per tree
    
    -- File paths
    trace_data = "timing_traces.csv",
    model_output = "model_weights.dat"
}

return settings