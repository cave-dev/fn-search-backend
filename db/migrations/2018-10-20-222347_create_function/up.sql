CREATE TABLE "function" (
	"id" bigserial NOT NULL,
	"repo_id" serial NOT NULL,
	"type_signature" TEXT NOT NULL,
	"return_type" TEXT,
	CONSTRAINT function_pk PRIMARY KEY ("id")
) WITH (
  OIDS=FALSE
);

ALTER TABLE "function" ADD CONSTRAINT "function_fk0" FOREIGN KEY ("repo_id") REFERENCES "repository"("id");
