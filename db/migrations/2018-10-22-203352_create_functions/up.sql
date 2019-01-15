CREATE TABLE "functions" (
  "id" bigserial NOT NULL,
  "repo_id" serial NOT NULL,
  "name" TEXT NOT NULL,
  "type_signature" TEXT NOT NULL,
  CONSTRAINT functions_pk PRIMARY KEY ("id"),
  CONSTRAINT function_repo UNIQUE (repo_id, name, type_signature)
) WITH (
  OIDS=FALSE
);

ALTER TABLE "functions" ADD CONSTRAINT "functions_fk0" FOREIGN KEY ("repo_id") REFERENCES "repositories"("id");
