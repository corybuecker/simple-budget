module Adjustments.Update exposing (adjustmentUrl, adjustmentsUrl, delete, deleteAdjustmentAndRefreshAdjustments, deleteAdjustmentTask, put, saveAdjustmentAndRefreshAdjustments, saveAdjustmentTask, saveNewAdjustmentTask, update)

import Accounts.Update exposing (refreshAccountsTask)
import Adjustments.Messages
import Adjustments.Models exposing (Adjustment)
import Adjustments.Utils exposing (adjustmentDecoder, adjustmentUpdatedDecoder, adjustmentsDecoder, encode)
import Adjustments.Views
import Http exposing (get, jsonBody, post, toTask)
import Json.Decode
import Model exposing (Model, Msg(..))
import Task exposing (Task)
import Url.Builder as Url


update : Adjustments.Messages.Msg -> Model -> ( Model, Cmd Model.Msg )
update msg model =
    case model.activeAdjustment of
        Just oldActiveAdjustment ->
            case msg of
                Adjustments.Messages.TitleUpdated newName ->
                    let
                        newActiveAdjustment =
                            { oldActiveAdjustment | title = newName }
                    in
                    ( { model | activeAdjustment = Just newActiveAdjustment }, Cmd.none )

                Adjustments.Messages.TotalUpdated newTotal ->
                    let
                        newActiveAdjustment =
                            case String.toFloat newTotal of
                                Just val ->
                                    { oldActiveAdjustment | total = val }

                                Nothing ->
                                    { oldActiveAdjustment | total = 0 }
                    in
                    ( { model | activeAdjustment = Just newActiveAdjustment }, Cmd.none )

                Adjustments.Messages.SaveAdjustment ->
                    ( model, saveAdjustmentAndRefreshAdjustments oldActiveAdjustment )

                Adjustments.Messages.DeleteAdjustment ->
                    ( model, deleteAdjustmentAndRefreshAdjustments oldActiveAdjustment )

        Nothing ->
            ( model, Cmd.none )


deleteAdjustmentAndRefreshAdjustments : Adjustments.Models.Adjustment -> Cmd Msg
deleteAdjustmentAndRefreshAdjustments model =
    Task.attempt AccountsFetched (Task.andThen refreshAccountsTask (deleteAdjustmentTask model))


saveAdjustmentAndRefreshAdjustments : Adjustments.Models.Adjustment -> Cmd Msg
saveAdjustmentAndRefreshAdjustments model =
    case model.id of
        0 ->
            Task.attempt AccountsFetched (Task.andThen refreshAccountsTask (saveNewAdjustmentTask model))

        _ ->
            Task.attempt AccountsFetched (Task.andThen refreshAccountsTask (saveAdjustmentTask model))


saveAdjustmentTask : Adjustments.Models.Adjustment -> Task Http.Error Adjustment
saveAdjustmentTask model =
    toTask (put (adjustmentUrl model.accountId model.id) (jsonBody (encode model)) adjustmentUpdatedDecoder)


deleteAdjustmentTask : Adjustments.Models.Adjustment -> Task Http.Error String
deleteAdjustmentTask model =
    toTask (delete (adjustmentUrl model.accountId model.id))


saveNewAdjustmentTask : Adjustments.Models.Adjustment -> Task Http.Error Adjustment
saveNewAdjustmentTask model =
    toTask (post (adjustmentsUrl model.accountId) (jsonBody (encode model)) adjustmentUpdatedDecoder)


adjustmentsUrl : Int -> String
adjustmentsUrl accountId =
    Url.crossOrigin "//localhost:4000"
        [ "api", "accounts", String.fromInt accountId, "adjustments" ]
        []


adjustmentUrl : Int -> Int -> String
adjustmentUrl accountId id =
    Url.crossOrigin "//localhost:4000"
        [ "api", "accounts", String.fromInt accountId, "adjustments", String.fromInt id ]
        []


put : String -> Http.Body -> Json.Decode.Decoder a -> Http.Request a
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


delete : String -> Http.Request String
delete url =
    Http.request
        { method = "DELETE"
        , headers = []
        , url = url
        , body = Http.emptyBody
        , expect = Http.expectString
        , timeout = Nothing
        , withCredentials = False
        }
