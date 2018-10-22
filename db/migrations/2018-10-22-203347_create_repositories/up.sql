CREATE TABLE "repositories" (
	"id" serial NOT NULL,
	"url" TEXT NOT NULL,
	CONSTRAINT repositories_pk PRIMARY KEY ("id")
) WITH (
  OIDS=FALSE
);
