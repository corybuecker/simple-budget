module Accounts.Update exposing (accountUrl, accountsUrl, fetchAccounts, put, refreshAccountsTask, saveAccountAndRefreshAccounts, saveAccountTask, update)

import Accounts.Messages
import Accounts.Models exposing (Account)
import Accounts.Utils exposing (accountDecoder, accountUpdatedDecoder, accountsDecoder, adjustmentDecoder, encode)
import Accounts.Views
import Http exposing (get, jsonBody, post, toTask)
import Json.Decode
import Model exposing (Model, Msg(..))
import Task exposing (Task)
import Url.Builder as Url


update : Accounts.Messages.Msg -> Model -> ( Model, Cmd Model.Msg )
update msg model =
    case msg of
        Accounts.Messages.NameUpdated newName ->
            let
                oldActiveAccount =
                    model.activeAccount

                newActiveAccount =
                    { oldActiveAccount | name = newName }
            in
                ( { model | activeAccount = newActiveAccount }, saveAccountAndRefreshAccounts newActiveAccount )

        _ ->
            ( model, Cmd.none )


fetchAccounts : Cmd Msg
fetchAccounts =
    Http.send AccountsFetched (get accountsUrl accountsDecoder)


saveAccountAndRefreshAccounts : Accounts.Models.Account -> Cmd Msg
saveAccountAndRefreshAccounts model =
    Task.attempt AccountsFetched (Task.andThen refreshAccountsTask (saveAccountTask model))


refreshAccountsTask : Account -> Task Http.Error (List Account)
refreshAccountsTask _ =
    toTask (get accountsUrl accountsDecoder)


saveAccountTask : Accounts.Models.Account -> Task Http.Error Account
saveAccountTask model =
    toTask (put (accountUrl model.id) (jsonBody (encode model)) accountUpdatedDecoder)


accountsUrl : String
accountsUrl =
    Url.crossOrigin "//localhost:4000"
        [ "api", "accounts" ]
        []


accountUrl : Int -> String
accountUrl id =
    Url.crossOrigin "//localhost:4000"
        [ "api", "accounts", String.fromInt id ]
        []


put : String -> Http.Body -> Json.Decode.Decoder Account -> Http.Request Account
put url body decoder =
    Http.request
        { method = "PUT"
        , headers = []
        , url = url
        , body = body
        , expect = Http.expectJson decoder
        , timeout = Nothing
        , withCredentials = False
        }
