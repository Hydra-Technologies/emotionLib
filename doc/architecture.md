# Dokumentation
Hier werde ich versuche eine Dokumentation über die Bibeliothek zu führen. Das hat den Grund, dass jedesmal wenn ich mir den Code angucke ich nicht mehr versteh, was er tut. Das ist kacke.

Also hier ein paar Dokumente zur Unterstützung.
# Datenbank
Hier ist ein Diagramm mit der Datenbank struktur

```mermaid
erDiagram

    katGroups{
        int id
        string name
        int numPflicht
        bool forEDay
    }
    kategorien{
        int id
        string name
        char einheit
        bool lauf
        int maxVers
        int digits_before
        int digits_after
        int kateGroupId
    }

    formVars{
        int katId
        char gesch
        double a
        double c
    }

    ageGroups{
        int age
        char gesch
        int gold
        int silber
    }

    schueler{
        int id
        int external_id
        string fname
        string lname
        string klasse
        char gesch
        int birth_year
        int age
        bool aufsicht
        int llTime
        String llkey
    }

    versuch{
        int id
        string aufsichtId
        int schuelerid
        int kategorieId
        double wert
        int punkte
        int mTime
        bool isReal
    }

    dosbKat{
        int age
        char gesch
        int katId
        double gold
        double silber
        double bronze
    }

    loginKeys{
        int aufsichtId
        string token
        int buildTime
    }

    kategorien }o--|| katGroups : "ist in"
    formVars }|..|| kategorien : "hat"
    schueler }o--|| ageGroups : "ist"
    versuch }o..|| schueler : versucht
    versuch }o--|| kategorien : "ist in" 
    ageGroups }o--o{ kategorien : "braucht für bjs"
    dosbKat }o--|| ageGroups : "hat"
    dosbKat }o--|| kategorien : "hat"
    loginKeys }o..|| schueler : "besitzt"
```

Die n:m Verbindung "braucht für bjs" wurde mit folgender Relation umgesetzt:
```mermaid
erDiagram
    bjsKat{
        int age
        char gesch
        int katId
    }
```

# Main
Die Hauptmodell ist das interact Module. In dem sind einige Funktionen mit denen man mit der Datenbank kommunizieren kann.
Diese werden für den Upload von Schuelern, Daten von Schuelern abrufen und Informationen zu Kategorien. Hier eine volle List der funktionen:

+ upload_schueler

+ get\_schueler
+ get_dosb_task_for_schueler
+ get_bjs_task_for_schueler
+ get_all_versuch_for_kat
+ get_top_versuch_by_kat
+ get_top_versuch_in_bjs
+ get_top_versuch_in_dosb
+ get_bjs_kat_groups
+ need_kat
+ add_versuch
+ get_versuch_by_id
+ set_is_real
+ get_all_kat
+ get_kategorie
+ calc_points
Diese brauchen fast immer eine Referenz zu einem SQLitePool.

# Manage
Hier sind funktionen die genutzt werden können um die Datenbank selber zu modifizieren. Dabei werden die folgenden Funktionen benu

