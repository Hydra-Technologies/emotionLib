SELECT ifnull(kategorieId, -1) as kategorie_id, ifnull(wert ,-1.0) as wert FROM 
-- get best Attempts of each student in each category
(SELECT schuelerId, kategorieId, ROUND(MIN(wert), 2) as wert FROM versuch -- For Sprint and Ausdauer
    INNER JOIN kategorien ON kategorieId = kategorien.id
    WHERE kategorien.kateGroupIdBJS IN (1, 4) AND isReal = true GROUP BY versuch.schuelerId, kategorien.id
UNION 
SELECT schuelerId, kategorieId, ROUND(MAX(wert), 2) as wert FROM versuch -- For Sprung and Wurf/Stoß
    INNER JOIN kategorien ON kategorieId = kategorien.id
    WHERE kategorien.kateGroupIdBJS IN (2, 3) AND isReal = true GROUP BY versuch.schuelerId, kategorien.id) as trys
WHERE schuelerId = ?
