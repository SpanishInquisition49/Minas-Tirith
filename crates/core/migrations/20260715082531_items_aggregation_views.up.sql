-- Add up migration script here
-- NOTE: view for authors
CREATE VIEW IF NOT EXISTS view_authors_aggregated AS SELECT
  ia.item_id,
  json_group_array(json_object(
    'id',
    a.id,
    'name',
    a.name,
    'given_name',
    a.given_name,
    'family_name',
    a.family_name,
    'slug',
    a.slug,
    'bio',
    a.bio
  )) AS authors
FROM
  authors AS a
INNER JOIN item_authors AS ia
  ON a.id = ia.author_id
GROUP BY
  ia.item_id;
-- NOTE: view for collections
CREATE VIEW IF NOT EXISTS view_collections_aggregated AS SELECT
  ic.item_id,
  json_group_array(json_object(
    'id',
    c.id,
    'name',
    c.name,
    'slug',
    c.slug
  )) AS collections
FROM
  collections AS c
INNER JOIN item_collections AS ic
  ON c.id = ic.collection_id
GROUP BY
  ic.item_id;
--NOTE: view for tags
CREATE VIEW IF NOT EXISTS view_tags_aggregated AS SELECT
  it.item_id,
  json_group_array(json_object(
    'id',
    t.id,
    'name',
    t.name,
    'slug',
    t.slug
  )) AS tags
FROM
  tags AS t
INNER JOIN item_tags AS it
  ON t.id = it.tag_id
GROUP BY
  it.item_id
