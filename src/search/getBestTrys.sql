SELECT ifnull(kategorieId, -1) as kategorie_id, ifnull(wert ,-1.0) as wert FROM 
-- get best Attempts of each student in each category
(SELECT schuelerId, kategorieId, ROUND(MIN(wert), 2) as wert FROM versuch -- For Sprint and Ausdauer
    INNER JOIN kategorien ON kategorieId = kategorien.id
    WHERE kategorien.kateGroupId IN (1, 4) GROUP BY versuch.schuelerId, kategorien.kateGroupId
UNION 
SELECT schuelerId, kategorieId, ROUND(MAX(wert), 2) as wert FROM versuch -- For Sprung and Wurf/Stoß
    INNER JOIN kategorien ON kategorieId = kategorien.id
    WHERE kategorien.kateGroupId IN (2, 3) GROUP BY versuch.schuelerId, kategorien.kateGroupId) as trys
WHERE schuelerId = ?
