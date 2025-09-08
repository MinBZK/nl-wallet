workspace "Name" "NL-Wallet" {

    !identifiers hierarchical

    model {
        u = person "User"
        ua = person "Wallet Technical Support"
        us = person "Wallet User Support"
        uaPid = person "PID issuer admin"
        uaPb = person "Issuer admin"

        ws = softwareSystem "NL-Wallet" {
            wab = group "NL-Wallet App containers" {
            walletApp = container "Wallet app" "" "Android/iOS" {
                appGui = component "App Frontend" "" "flutter (dart)"  
                appCore = component "App core component" "" "rust"
                appPlatform = component "Platform support" "native functions" "rust"
            }
            appDb = container "App database" "" "sqlite" {
                tags "Database"
            }      
            secureElement = container "Secure Element" "" "Secure Enclave, Android Keystore system" 
            }
            
            wb = group "NL-Wallet backend containers" {
                walletBackend = container "WalletBackend (WP)" "Wallet backend" "axum (rust)" {
                    hsmInstructionClient = component "Assisted wallet Instructions endpoint (/instructions)"
                    walletAccountManager = component "Wallet Accounts endpoint (enroll/create)"
                    walletStatusManager = component "Wallet Status endpoint (/status)"
                }
                updateServer = container "UpdateServer" "Serve app update policy" "nginx (static)" {
                    updatePolicy = component "Policy configuration" "" "[update policy] section in .toml file" { 
                        tags "File" 
                    }
                }

                statusList = container "WUA status list" "" "Static content (TSL)"
                configurationServer = container "ConfigurationServer" "Serve app config file" "nginx (static)"               
                db = container "WP database (accounts, WUA status)" "" "postgress" {
                    tags "Database"
                }                
            }
            walletHsm = container "HSM device" "dedicated cryptographic hardware" 
            //walletAdminPortal = container "Self Serivce portal" "Selfservice and assitance" "web front-end"
        }
        platformServicesApple = softwareSystem "Apple AppAttest"{
          tags "External"            
        }
        platformServices = softwareSystem "Apple AppAttest / Google Play Integrity"{
          tags "External"            
        }
        
        hc = softwareSystem "BRP-V"

        verifier = softwareSystem "Verifier" { 
            tags "External" 
            ov = container "OV-component"{
                verifying = component "Disclosure endpoint"
            }

            rpApp = container "Relying Party application"
        }
        issuerPb = softwareSystem "(Pub/Q)EAA Issuer" { 
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
            mockUserStorage = container "Demo user storage" "" "Static files" { 
                tags "Database" 
            }
            authServer = container "Authorization server" "" "OIDC/SAML proxy" 
        }

        haalCentraal = softwareSystem "BRP V" {
            tags "External"
        }

        digid = softwareSystem "DigiD" "OIDC/SAML proxy" {
            tags "External"
        }

        issuerPid -> digid "User authentication"
        u -> ws "Uses"
        // u -> ws.walletAdminPortal "Self service"
        u -> ws.walletApp "Uses"
        u -> ws.walletApp.appGui "Has interactions"
        ua -> ws "Manage system" 
        us -> ws "Perform user support actions" 
        ua -> ws.configurationServer "Maintain runtime config"
        ua -> ws.updateServer.updatePolicy "Maintain updatepolicy"

        ws -> platformServices "Request/verify app- and keyattestations" 
        ws -> digid "Start user authentication (onboarding and recovery)"
        ws -> verifier.ov "Present data"
        //issuerPb -> ws "Issue attestations" 
        ws.walletApp -> platformServices "Request App/key attestation (Apple AppAttest)"
        ws.walletBackend -> platformServices "Verify App attestation (Google Play Integrity)"
        ws.walletApp -> digid "Start authentication for activation/recovery"
        ws.walletBackend -> ws.db "Reads from and writes to"

        ws.walletApp.appCore -> ws.walletApp.appGui "Exchange information from core to GUI"
        ws.walletApp.appGui -> ws.walletApp.appCore "Exchange information from GUI to core"
        ws.walletApp.appCore -> ws.walletApp.appPlatform "Use platform routines (iOS/Android)"
        ws.walletApp.appCore -> ws.updateServer "Get update policies"
        ws.walletApp.appCore -> ws.walletBackend.walletAccountManager "WP operations (account, HSM instructions)"
        ws.walletApp.appCore -> ws.configurationServer "Get runtime configuration"
        ws.walletApp.appCore -> ws.walletBackend.hsmInstructionClient "HSM-assisted operation"
        ws.walletApp.appCore -> ws.appDb "Store/retrieve attestations, logs, configuration"
        ws.walletApp.appCore -> ws.secureElement "Manage keys, signing ops"

        //PID issuer specific
        ws.walletApp.appCore -> issuerPid.authServer "Wallet activation and PID issuance"

        ws.walletBackend -> ws.walletHsm "Call HSM for assisted operation"
        ws.walletBackend -> ws.statusList "Publish WUA statuslist" 


        ws.walletApp -> issuerPb.vvPbi "Perform disclosure based issuance, Retrieve Status List"

        ws.walletApp -> verifier "Disclose attributes to verifier"

        ws.walletBackend.walletAccountManager -> ws.walletBackend.walletStatusManager "Update WUA status"
        ws.walletBackend.hsmInstructionClient -> ws.walletHsm "Process HSM instruction"
        us -> ws.walletBackend.walletAccountManager "Manage wallet instances"
        issuerPid -> ws.statusList "Get WUA status" 

        ws.walletApp.appCore -> issuerPid.statusList "Get attestation status list (PID)" 
        //verifier.ov -> issuerPid.statusList "Get attestation status list (PID)"
        uaPid -> issuerPid.vvPid.statusManager "Update PID attestation status" 
        issuerPid.vvPid.statusManager -> issuerPid.statusList "Publish Status List" 

        issuerPid.vvPid.statusManager -> issuerPid.pidStatusDb "Persist/retrieve attestation status"
        issuerPid.vvPid.issuing -> issuerPid.vvPid.statusManager "Persist attestation status"
        issuerPid.vvPid.issuing -> issuerPid.authServer "Get authenticated BSN"


        issuerPid.vvPid -> issuerPid.mockUserStorage "Retrieve PID attestation data"
        //issuerPid.authServer -> digid "Retrieve authentication result"

        //issuerPid.pidIssuer -> issuerPid.mockUserStorage "Fetch PID-Attributes"
        ws.walletApp.appCore -> issuerPid.vvPid  "Retrieve PID / Disclose WUA + PoA" 
        //issuerPid.vvPid.issuing  -> issuerPid.pidIssuer "Retrieve attestation data"
        issuerPid.vvPid.issuing  -> issuerPid.mockUserStorage "Retrieve attestation data"
        //issuerPid.mockUserStorage -> haalCentraal "Call BRP V"

        ws.walletApp.appCore -> issuerPb "Perform disclosure based issuance, retrieve Status List" 
        ws.walletApp.appCore -> verifier "Perform disclosure of attributes" 
        ws.walletApp.appCore -> issuerPb.statusList "Get attestation status list" 
        //verifier.ov -> issuerPb.statusList "Get attestation status list"

        issuerPb.vvPbi.issuing -> issuerPb.ds "Retrieve attestation data for disclosed attestation"
        issuerPb.vvPbi.statusManager -> issuerPb.statusDb "Persist/retrieve attestation status"
        issuerPb.vvPbi.issuing -> issuerPb.vvPbi.statusManager "Persist attestation status"
        uaPb -> issuerPb.vvPbi.statusManager "Update attestation status" 
        issuerPb.vvPbi.statusManager -> issuerPb.statusList "Publish Status List" 


        verifier.rpApp -> verifier.ov.verifying "Disclosure session operations"
    }

    views {
        systemContext ws "AD1NL-Wallet" {
            include u ws verifier issuerPb issuerPid platformServices digid ua us
        }

        systemContext ws "B1PID-Issuer" {
            include u issuerPid ws verifier
        }

        container ws "D2NL-WalletSystem" {
            include * platformServices
        }

        component ws.walletBackend "GD2NL-walletBackend" {
            include * 
            
        }

        component ws.walletApp "HD2NL-WalletApp" {
            include * verifier
        }

        systemContext issuerPb "ID3IssuerSoftwareSystem" {
            include *
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

        properties {
            "structurizr.sort" "key"
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
                fontSize 32
            }
            element "Container" {
                background #2b81e9
                shape RoundedBox
                fontSize 32
            }
            element "Component" {
                background #1056ab
                shape Component
            }

            element "Database" {
                shape cylinder
            }
            element "DatabaseS" {
                shape cylinder
                fontSize 25
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
                fontSize 26

            }

            relationship "Relationship" {
                fontSize 28
            }
        }
    }

    configuration {
        scope none
    }

}