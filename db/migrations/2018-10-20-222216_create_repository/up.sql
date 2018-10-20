CREATE TABLE "repository" (
	"id" serial NOT NULL,
	"url" TEXT NOT NULL,
	CONSTRAINT repository_pk PRIMARY KEY ("id")
) WITH (
  OIDS=FALSE
);
