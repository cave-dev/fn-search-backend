-- drop materialized view
DROP MATERIALIZED VIEW repository_function_mat_view CASCADE;

-- drop unique name constraint
ALTER TABLE "repositories" DROP CONSTRAINT "repositories_name_unique";

-- revert unique constraint
ALTER TABLE "repositories" ADD CONSTRAINT "repositories_name_unique" UNIQUE (name);

-- remove version column
ALTER TABLE "repositories" DROP COLUMN ver;

-- recreate old materialized view
CREATE MATERIALIZED VIEW repository_function_mat_view AS
  SELECT r.id as repo_id, r.name as repo_name, r.url as repo_url,
         f.id as func_id, f.name as func_name, f.type_signature as func_type_sig
  FROM repositories AS r, functions AS f
  WHERE r.id = f.repo_id;

-- crate index on function id
CREATE UNIQUE INDEX repo_func_mat_view_func_id_index
  ON repository_function_mat_view (func_id);
