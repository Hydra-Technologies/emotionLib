-- calculate the dosb medal
SELECT schuelerId as schueler_id, versuchId as versuch_id, versuch.kategorieId as kategorie_id, kategorien.kateGroupId as kat_group_id, versuch.punkte as bjs_punkte, 
    IIF(silber<gold, 
        IIF(versuch.wert < bronze-0.01, 0, IIF(versuch.wert < silber-0.01, 1, IIF(versuch.wert < gold-0.01, 2, 3))),
        IIF(versuch.wert > bronze+0.01, 0, IIF(versuch.wert > silber+0.01, 1, IIF(versuch.wert > gold+0.01, 2, 3)))
    ) as dosb_abzeichen 
    FROM versuch
INNER JOIN
    -- get best Attempts of each student in each category
    (SELECT versuch.id as versuchId, MIN(wert) as wert FROM versuch -- For Sprint and Ausdauer
        INNER JOIN kategorien ON kategorieId = kategorien.id
        WHERE kategorien.kateGroupId IN (1, 4) AND isReal = true GROUP BY versuch.schuelerId, kategorien.kateGroupId
    UNION 
    SELECT versuch.id as versuchId, MAX(wert) as wert FROM versuch -- For Sprung and Wurf/Stoß
        INNER JOIN kategorien ON kategorieId = kategorien.id
        WHERE kategorien.kateGroupId IN (2, 3) AND isReal = true GROUP BY versuch.schuelerId, kategorien.kateGroupId) as bestTrys
ON bestTrys.versuchId = versuch.id
-- get the age of the student to determin the medal values
INNER JOIN schueler ON schueler.id = versuch.schuelerId
-- get the medal values for each versuch
LEFT JOIN dosbKat ON versuch.kategorieId = dosbKat.katId AND schueler.gesch = dosbKat.gesch AND schueler.age = dosbKat.age
-- NO DATA FOR STUDENT IN dosbKat STUDENT WILL BE DROPED
INNER JOIN kategorien ON versuch.kategorieId = kategorien.id
ORDER BY schueler_id