workspace "Name" "NL-Wallet" {

    !identifiers hierarchical

    model {
        u = person "User"
        ua = person "Wallet Administrator"
        uaPid = person "PID issuer admin"
        uaPb = person "Issuer admin"

        ws = softwareSystem "NL-Wallet Solution" {
            walletApp = container "Wallet app" "" "Android/iOS" {
                appGui = component "App Frontend" "flutter (dart)"  
                appFrb = component "Flutter-Rust-Bridge" "dart/rust binding"
                appCore = component "App core component" "" "rust"
                appPlatform = component "Platform support" "native functions" "rust"
                db = component "App database" "" "sqlite" {
                    tags "Database"
                }      
            }
            wb = group "NL-Wallet backend containers" {
                walletProvider = container "walletProvider" "Wallet backend" "axum (rust)" {
                    hsmInstructionClient = component "Assisted wallet Instructions endpoint (/instructions)"
                    walletAccountManager = component "Wallet Accounts endpoint (enroll/create)"
                    walletStatusManager = component "Wallet Status endpoint (/status)"
                }
                updateServer = container "UpdateServer" "Serve app update policy" "axum (rust)" {
                    updatePolicy = component "Policy configuration" "" "[update policy] section in .toml file" { 
                        tags "File" 
                    }
                }

                statusList = container "WUA status list" "" "Static content (TSL)"
                configurationServer = container "ConfigurationServer" "Serve app config file" "axum (rust)"               
                db = container "WP database (accounts, WUA status)" "" "postgress" {
                    tags "Database"
                }                
            }
            walletHsm = container "HSM device" "dedicated cryptographic hardware" 
            //walletAdminPortal = container "Self Serivce portal" "Selfservice and assitance" "web front-end"
        }

        hc = softwareSystem "BRP-V"

        verifier = softwareSystem "Verifier (OV)" { 
            tags "External" 
            ov = container "OV-component"{
                verifying = component "Disclosure endpoint"
            }

            rpApp = container "Relying Party application"
        }
        issuerPb = softwareSystem "Issuer - Disclosure based" { 
            tags "External" 

            vvPbi = container "VV for Disclosure based issuance" {
                statusManager = component "Attestation Status manager" ""  "Rust (endpoint)"
                issuing = component "Disc. based issuing endpoint" "" "Rust (endpoint)" 
            }
            statusDb = container "attestation-status storage" "" "postgress DB" { 
                    tags "Database"
            }

            ds = container "Issuer attestation data source"
            statusList = container "Attestation status list" "" "Static content (TSL)"
        }

        issuerPid = softwareSystem "PID Issuer" { 
            tags "External" 

            vvPid = container "VV for PID issuer" "" "Rust app" {
                statusManager = component "Attestation Status manager" ""  "Rust (endpoint)"
                issuing = component "Issuing endpoint" "" "Rust (endpoint)" 
            }
            pidStatusDb = container "attestation-status storage" "" "postgress DB" { 
                    tags "Database"
            }
            //pidIssuer = container "PID-issuer business logic" "" "Rust app"
            statusList = container "PID attestation status list" "" "Static content (TSL)"
            gbaHcConverter = container "GBA_hc_converter" "" "Rust app"
            authServer = container "Authorization server (RDO-max)" "" "Python" 
        }

        haalCentraal = softwareSystem "BRP V" {
            tags "External"
        }

        digid = softwareSystem "DigiD" {
            tags "External"
        }

        u -> ws "Uses"
        // u -> ws.walletAdminPortal "Self service"
        u -> ws.walletApp "Uses"
        u -> ws.walletApp.appGui "Has interactions"
        ua -> ws.updateServer.updatePolicy "Maintain updatepolicy"
        ua -> ws.configurationServer "Maintain runtime config"
        //ua -> ws "Manage system" 

        ws -> verifier.ov "Present data"
        issuerPb -> ws "Issue attestations" 
        ws.walletProvider -> ws.db "Reads from and writes to"

        ws.walletApp.appFrb -> ws.walletApp.appGui "Exchange information from frb to GUI"
        ws.walletApp.appGui -> ws.walletApp.appFrb "Exchange information from GUI to frb"
        ws.walletApp.appFrb -> ws.walletApp.appCore "Exchange information from core with frb"
        ws.walletApp.appCore -> ws.walletApp.appFrb "Exchange information from frb to core"
        ws.walletApp.appCore -> ws.walletApp.appPlatform "Use platform routines (iOS/Android)"
        ws.walletApp.appCore -> ws.updateServer "Get update policies"
        ws.walletApp.appCore -> ws.walletProvider.walletAccountManager "WP operations (account, HSM instructions)"
        ws.walletApp.appCore -> ws.configurationServer "Get runtime configuration"
        ws.walletApp.appCore -> ws.walletProvider.hsmInstructionClient "HSM-assisted operation"
        ws.walletApp.appCore -> ws.walletApp.db "Store/retrieve attestations, logs, configuration"

        //PID issuer specific
        ws.walletApp.appCore -> issuerPid.authServer "Authentication for PID-issuance"

        ws.walletProvider -> ws.walletHsm "Call HSM for assisted operation"
        ws.walletProvider -> ws.statusList "Publish WUA statuslist" 


        ws.walletApp -> issuerPb.vvPbi "Interactions with issuer, (disclose based) issuance"

        ws.walletApp -> verifier "Disclose attributes to verifier"

        ws.walletProvider.walletAccountManager -> ws.walletProvider.walletStatusManager "Update WUA status"
        ws.walletProvider.hsmInstructionClient -> ws.walletHsm "Process HSM instruction"
        ua -> ws.walletProvider.walletAccountManager "Revoke wallet"
        issuerPid -> ws.statusList "Get WUA status (not in scope WP3)" 

        ws.walletApp.appCore -> issuerPid "interactions with PID issuer" 
        ws.walletApp.appCore -> issuerPid.statusList "Get PID status list" 
        verifier.ov -> issuerPid.statusList "Get PID status list"
        uaPid -> issuerPid.vvPid.statusManager "Update PID attestation status" 
        issuerPid.vvPid.statusManager -> issuerPid.statusList "Publish Status List" 

        issuerPid.vvPid.statusManager -> issuerPid.pidStatusDb "Persist/retrieve attestation status"
        issuerPid.vvPid.issuing -> issuerPid.vvPid.statusManager "Persist attestation status"
        issuerPid.vvPid.issuing -> issuerPid.authServer "Get authenticated BSN"


        issuerPid.vvPid -> issuerPid.gbaHcConverter "Retrieve PID attestation data"
        issuerPid.authServer -> digid "Retrieve authentication result"

        //issuerPid.pidIssuer -> issuerPid.gbaHcConverter "Fetch PID-Attributes"
        ws.walletApp.appCore -> issuerPid.vvPid  "Retrieve PID / Disclose WUA (formerly WTE) + PoA" 
        //issuerPid.vvPid.issuing  -> issuerPid.pidIssuer "Retrieve attestation data"
        issuerPid.vvPid.issuing  -> issuerPid.gbaHcConverter "Retrieve attestation data"
        issuerPid.gbaHcConverter -> haalCentraal "Call BRP V"

        ws.walletApp.appCore -> issuerPb "interactions with EAA issuer" 
        ws.walletApp.appCore -> issuerPb.statusList "Get attestation status list" 
        verifier.ov -> issuerPb.statusList "Get attestation status list"

        issuerPb.vvPbi.issuing -> issuerPb.ds "Retrieve attestation data for disclosed attestation"
        issuerPb.vvPbi.statusManager -> issuerPb.statusDb "Persist/retrieve attestation status"
        issuerPb.vvPbi.issuing -> issuerPb.vvPbi.statusManager "Persist attestation status"
        uaPb -> issuerPb.vvPbi.statusManager "Update attestation status" 
        issuerPb.vvPbi.statusManager -> issuerPb.statusList "Publish Status List" 


        verifier.rpApp -> verifier.ov.verifying "Disclosure session operations"
    }

    views {
        systemContext ws "AD1NL-Wallet" {
            include u ws verifier issuerPb
        }

        systemContext ws "B1PID-Issuer" {
            include u issuerPid ws verifier
        }

        systemContext ws "C2NL-Wallet" {
            include u ws verifier issuerPid issuerPb
        }

        container ws "D2NL-WalletSystem" {
            include * 
        }

        container ws "E3NL-WalletSystem" {
            include * verifier
        }

        container verifier "FRelyingParty"{
            include * ws
        }

        component ws.walletProvider "GD2NL-WalletProvider" {
            include * 
            
        }

        component ws.walletApp "HD2NL-WalletApp" {
            include * 
        }

        systemContext issuerPb "ID3IssuerSoftwareSystem" {
            include *
        }

        container issuerPb "JD4IssuerSoftwareSystem" {
            include * ws 
        }

        container issuerPid "KD4PID_IssuerSoftwareSystem" {
            include * ws uaPid digid
        }

        component issuerPid.vvPid "MD5PID_IssuerVV" {
            include * uaPid
        }

        component issuerPb.vvPbi "ND5PID_IssuerPB" {
            include * uaPb
        }

        component ws.updateServer "O5UpdateServer" {
            include *
        }

        properties {
            "structurizr.sort" "key"
        }

        dynamic ws {
            title "Wallet Migration"
            ws.walletApp -> ws.walletProvider "Disclose recovery code (after Activate new Wallet)"
            ws.walletProvider -> ws.walletApp "Recovery possible (y/n)"
            ws.walletApp -> u "Recover? (if possible)"
            u -> ws.walletApp "Yes Proceed with recovery "
            ws.walletApp -> ws.walletProvider ""
        }
      
        styles {
            element "Element" {
                color #ffffff
            }
            element "Person" {
                background #09326b
                shape person
            }
            element "Software System" {
                background #2b81e9
                shape Window
                fontSize 40
            }
            element "Container" {
                background #2b81e9
                shape RoundedBox
                fontSize 40
            }
            element "Component" {
                background #1056ab
                shape Component
            }

            element "Database" {
                shape cylinder
            }
            element "File" {
                shape Folder
            }

            element "newComponent" {
                background #88DCaa
            }

            element "NewDB" {
                shape cylinder
                background #88DCaa
            }
            element "External" {
                background #aaaaaa
            }

            relationship "Relationship" {
                fontSize 35
            }
        }
    }

    configuration {
        scope none
    }

}