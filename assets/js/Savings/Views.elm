module Savings.Views exposing (editView, renderSaving, renderSavings)

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Savings.Messages exposing (..)
import Savings.Models exposing (Saving)


editView : Saving -> Html Msg
editView model =
    div []
        [ input [ type_ "text", value model.title, onInput TitleUpdated ] []
        , input [ type_ "text", value (String.fromFloat model.amount), onInput AmountUpdated ] []
        , button [ onClick SaveSaving ] [ text "Save" ]
        , button [ onClick DeleteSaving ] [ text "Delete" ]
        ]


renderSavings : List Saving -> Html Msg
renderSavings savings =
    div []
        [ button [ class "btn btn-primary", onClick CreateSaving ] [ text "New Saving" ]
        , div [ class "row" ] (List.map renderSaving savings)
        ]


renderSaving : Saving -> Html Msg
renderSaving saving =
    div [ class "col-sm-6" ]
        [ div [ class "card" ]
            [ div [ class "card-body" ]
                [ h5 [ class "card-title", onClick (OpenSavingEditor saving) ] [ text saving.title ]
                , div [] [ text (String.fromFloat saving.amount) ]
                ]
            ]
        ]
