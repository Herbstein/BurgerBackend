workspace "Burger Backend" "" {

    model {
        user = person "User" "Burger Lover"
        burger = softwareSystem "Burger Backend" {
            webapp = container "Backend" "API server"
            database = container "Database Cluster" "Cluster of databases, size depending on scale"
            front = container "Frontend" "SPA"

            db_cache = container "Database Cache" "Cache database operations to avoid roundtrip and re-calculation"
        }

        user -> front "Finds restaurants and posts reviews"
        
        front -> webapp "Request and update data when navigating page"
        webapp -> front "Stream real-time data to frontend"

        webapp -> db_cache "Read data and write data"
        db_cache -> database "Update database and keep consistency"

    }

    views {
        container burger {
            include *
            autoLayout lr
        }

        systemContext burger {
            include *
            autoLayout lr
        }
    }

}
