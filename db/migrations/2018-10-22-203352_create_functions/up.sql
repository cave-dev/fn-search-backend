CREATE TABLE "functions" (
  "id" bigserial NOT NULL,
  "repo_id" serial NOT NULL,
  "name" TEXT NOT NULL,
  "type_signature" TEXT NOT NULL,
  CONSTRAINT functions_pk PRIMARY KEY ("id")
  ) WITH (
  OIDS=FALSE
);

ALTER TABLE "functions" ADD CONSTRAINT "functions_fk0" FOREIGN KEY ("repo_id") REFERENCES "repositories"("id");
