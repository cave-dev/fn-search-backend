CREATE MATERIALIZED VIEW repository_function_mat_view AS
  SELECT r.id as repo_id, r.name as repo_name, r.url as repo_url,
         f.id as func_id, f.name as func_name, f.type_signature as func_type_sig
  FROM repositories AS r, functions AS f
  WHERE r.id = f.repo_id;
